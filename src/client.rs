use crate::{
    event::{Builder as EventBuilder, Event},
    headers::{HeaderError, Headers},
    subject::{Subject, SubjectError, TERMINATED_PREFIX},
    Config, MessageStream, Messages, NatsClient,
};
use anyhow::anyhow;

use async_nats::{
    jetstream::{
        consumer::{self, AckPolicy, DeliverPolicy, PullConsumer, PushConsumer},
        AckKind, Context, Message,
    },
    Client as AsyncNatsClient, ConnectError, Error, Event as NatsEvent,
};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, warn};

#[derive(Clone)]
pub struct Client {
    inner: AsyncNatsClient,
    jetstream: Context,
    config: Config,
}

impl Client {
    pub async fn new(config: Config) -> Result<Self, ConnectError> {
        let creds: &str = config.creds.as_ref();
        let client = async_nats::ConnectOptions::with_credentials_file(creds.into())
            .await?
            .event_callback(|event| async move {
                let error = match event {
                    NatsEvent::ServerError(err) => anyhow!(err),
                    NatsEvent::ClientError(err) => anyhow!(err),
                    event => {
                        warn!(%event, "nats connection status");
                        return;
                    }
                };

                error!(%error);
                if let Err(err) = svc_error::extension::sentry::send(Arc::new(error)) {
                    error!(%err);
                }
            })
            .connect(&config.url)
            .await?;

        let jetstream = async_nats::jetstream::new(client.clone());

        Ok(Self {
            inner: client,
            jetstream,
            config,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error("failed to publish message: `{0}`")]
    PublishFailed(Error),
    #[error("failed to ack message: `{0}`")]
    AckFailed(Error),
}

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("config for subscription is not found")]
    SubscribeConfigNotFound,
    #[error("failed to get stream: `{0}`")]
    GettingStreamFailed(Error),
    #[error("failed to get consumer: `{0}`")]
    GettingConsumerFailed(Error),
    #[error("failed to create stream of messages: `{0}`")]
    StreamCreationFailed(Error),
    #[error("failed to create ephemeral consumer: `{0}`")]
    EphemeralConsumerCreationFailed(Error),
}

#[derive(Debug, thiserror::Error)]
pub enum TermMessageError {
    #[error(transparent)]
    InvalidHeader(#[from] HeaderError),
    #[error(transparent)]
    InvalidSubject(#[from] SubjectError),
    #[error(transparent)]
    PublishError(#[from] PublishError),
    #[error("failed to term message: `{0}`")]
    AckTermFailed(Error),
}

#[async_trait::async_trait]
impl NatsClient for Client {
    async fn publish(&self, event: &Event) -> Result<(), PublishError> {
        self.jetstream
            .publish_with_headers(
                event.subject().to_string(),
                event.headers().to_owned().into(),
                event.payload().to_owned().into(),
            )
            .await
            .map_err(PublishError::PublishFailed)?
            .await
            .map_err(PublishError::AckFailed)?;

        Ok(())
    }

    /// Returns a stream of messages for Durable Pull Consumer.
    async fn subscribe_durable(&self) -> Result<MessageStream, SubscribeError> {
        let config = self
            .config
            .subscribe_durable
            .as_ref()
            .ok_or(SubscribeError::SubscribeConfigNotFound)?;

        let stream = self
            .jetstream
            .get_stream(&config.stream)
            .await
            .map_err(SubscribeError::GettingStreamFailed)?;

        let consumer: PullConsumer = stream
            .get_consumer(&config.consumer)
            .await
            .map_err(SubscribeError::GettingConsumerFailed)?;

        let stream = consumer
            .stream()
            .max_messages_per_batch(config.batch)
            .heartbeat(config.idle_heartbeat)
            .messages()
            .await
            .map_err(SubscribeError::StreamCreationFailed)?;

        Ok(MessageStream(stream))
    }

    /// Returns a stream of messages for Ephemeral Push Consumer.
    async fn subscribe_ephemeral(
        &self,
        subject: Subject,
        deliver_policy: DeliverPolicy,
        ack_policy: AckPolicy,
    ) -> Result<Messages, SubscribeError> {
        let config = self
            .config
            .subscribe_ephemeral
            .as_ref()
            .ok_or(SubscribeError::SubscribeConfigNotFound)?;

        let stream = self
            .jetstream
            .get_stream(&config.stream)
            .await
            .map_err(SubscribeError::GettingStreamFailed)?;

        let consumer: PushConsumer = stream
            .create_consumer(consumer::push::Config {
                deliver_subject: self.inner.new_inbox(),
                filter_subject: subject.to_string(),
                ack_policy,
                deliver_policy,
                ..Default::default()
            })
            .await
            .map_err(SubscribeError::EphemeralConsumerCreationFailed)?;

        let messages = consumer
            .messages()
            .await
            .map_err(SubscribeError::StreamCreationFailed)?;

        Ok(messages)
    }

    async fn terminate(&self, message: &Message) -> Result<(), TermMessageError> {
        let headers = Headers::try_from(message.headers.clone().unwrap_or_default())?;
        let old_subject = Subject::from_str(&message.subject)?;
        let new_subject = Subject::new(
            format!("{}.{}", TERMINATED_PREFIX, old_subject.prefix()),
            old_subject.classroom_id(),
            old_subject.entity_type().into(),
        );
        let event = EventBuilder::new(
            new_subject,
            message.payload.to_vec(),
            headers.event_id().clone(),
            headers.sender_id().clone(),
        )
        .build();

        self.publish(&event).await?;

        message
            .ack_with(AckKind::Term)
            .await
            .map_err(TermMessageError::AckTermFailed)
    }
}
