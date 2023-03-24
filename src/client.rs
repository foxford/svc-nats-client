use crate::{Event, MessageStream, NatsClient};
use async_nats::{
    jetstream::{consumer::PullConsumer, Context},
    ConnectError, Error, Event as NatsEvent,
};
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
}
