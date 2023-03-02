use crate::{Event, MessageStream, NatsClient};
use async_nats::{
    jetstream::{consumer::PullConsumer, Context},
    Event as NatsEvent,
};
use std::io;
use tracing::{error, warn};

#[derive(Clone)]
pub struct Client {
    jetstream: Context,
}

pub async fn new(url: &str, creds: &str) -> io::Result<Client> {
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
    #[error("failed to publish message")]
    PublishFailed(String),
    #[error("failed to ack message")]
    AckFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("failed to get stream")]
    GettingStreamFailed(String),
    #[error("failed to get consumer")]
    GettingConsumerFailed(String),
    #[error("failed to create stream of messages")]
    StreamCreationFailed(String),
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
            .map_err(|e| PublishError::PublishFailed(e.to_string()))?
            .await
            .map_err(|e| PublishError::AckFailed(e.to_string()))?;

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
            .map_err(|e| SubscribeError::GettingStreamFailed(e.to_string()))?;

        let consumer: PullConsumer = stream
            .get_consumer(consumer)
            .await
            .map_err(|e| SubscribeError::GettingConsumerFailed(e.to_string()))?;

        let stream = consumer
            .messages()
            .await
            .map_err(|e| SubscribeError::StreamCreationFailed(e.to_string()))?;

        Ok(MessageStream(stream))
    }
}
