use crate::{
    event::Event,
    headers::{HeaderError, Headers},
    subject::{Subject, SubjectError, TERMINATED_PREFIX},
    MessageStream, NatsClient,
};
use async_nats::{
    jetstream::{consumer::PullConsumer, AckKind::Term, Context, Message},
    ConnectError, Error, Event as NatsEvent,
};
use std::str::FromStr;
use tracing::{error, warn};

#[derive(Clone)]
pub struct Client {
    jetstream: Context,
}

pub async fn new(url: &str, creds: &str) -> Result<Client, ConnectError> {
    let client = async_nats::ConnectOptions::with_credentials_file(creds.into())
        .await?
        .event_callback(|event| async move {
            match event {
                NatsEvent::ServerError(error) => {
                    error!(%error, "server error occurred");
                }
                NatsEvent::ClientError(error) => {
                    error!(%error, "client error occurred");
                }
                event => {
                    warn!(%event, "event occurred")
                }
            }
        })
        .connect(url)
        .await?;

    let jetstream = async_nats::jetstream::new(client);

    Ok(Client { jetstream })
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
    #[error("failed to get stream: `{0}`")]
    GettingStreamFailed(Error),
    #[error("failed to get consumer: `{0}`")]
    GettingConsumerFailed(Error),
    #[error("failed to create stream of messages: `{0}`")]
    StreamCreationFailed(Error),
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
                event.subject.to_string(),
                event.headers.to_owned().into(),
                event.payload.to_owned().into(),
            )
            .await
            .map_err(PublishError::PublishFailed)?
            .await
            .map_err(PublishError::AckFailed)?;

        Ok(())
    }

    async fn subscribe(
        &self,
        stream: &str,
        consumer: &str,
    ) -> Result<MessageStream, SubscribeError> {
        let stream = self
            .jetstream
            .get_stream(stream)
            .await
            .map_err(SubscribeError::GettingStreamFailed)?;

        let consumer: PullConsumer = stream
            .get_consumer(consumer)
            .await
            .map_err(SubscribeError::GettingConsumerFailed)?;

        let stream = consumer
            .messages()
            .await
            .map_err(SubscribeError::StreamCreationFailed)?;

        Ok(MessageStream(stream))
    }

    async fn term_message(&self, message: Message) -> Result<(), TermMessageError> {
        let headers = Headers::try_from(message.headers.clone().unwrap_or_default())?;
        let old_subject = Subject::from_str(&message.subject)?;
        let old_prefix = old_subject.prefix.clone();
        let new_subject = old_subject.prefix(format!("{TERMINATED_PREFIX}.{old_prefix}"));
        let event = Event::new(
            new_subject,
            message.payload.to_vec(),
            headers.event_id().clone(),
            headers.sender_id().clone(),
        );

        self.publish(&event).await?;

        message
            .ack_with(Term)
            .await
            .map_err(TermMessageError::AckTermFailed)
    }
}
