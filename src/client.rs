use crate::{headers::HeaderMap, MessageStream, NatsClient};
use async_nats::{
    jetstream::{consumer::PullConsumer, Context},
    Event,
};
use async_trait::async_trait;
use std::io;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Clone)]
pub struct Client {
    pub jetstream: Context,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to publish message")]
    PublishFailed(String),
    #[error("failed to ack message")]
    AckFailed(String),
    #[error("failed to get stream")]
    GettingStreamFailed(String),
    #[error("failed to get consumer")]
    GettingConsumerFailed(String),
    #[error("failed to create stream of messages")]
    StreamCreationFailed(String),
}

pub async fn new(url: &str, creds: &str) -> io::Result<Client> {
    let client = async_nats::ConnectOptions::with_credentials_file(creds.into())
        .await?
        .event_callback(|event| async move {
            match event {
                Event::ServerError(error) => {
                    error!(%error, "server error occurred");
                }
                Event::ClientError(error) => {
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

#[async_trait]
impl NatsClient for Client {
    async fn publish(
        &self,
        subject: String,
        payload: Vec<u8>,
        headers: Option<HeaderMap>,
    ) -> Result<(), Error> {
        self.jetstream
            .publish_with_headers(subject, headers.unwrap_or_default().into(), payload.into())
            .await
            .map_err(|e| Error::PublishFailed(e.to_string()))?
            .await
            .map_err(|e| Error::AckFailed(e.to_string()))?;

        Ok(())
    }

    async fn subscribe(&self, stream: &str, consumer: &str) -> Result<MessageStream, Error> {
        let stream = self
            .jetstream
            .get_stream(stream)
            .await
            .map_err(|e| Error::GettingStreamFailed(e.to_string()))?;

        let consumer: PullConsumer = stream
            .get_consumer(consumer)
            .await
            .map_err(|e| Error::GettingConsumerFailed(e.to_string()))?;

        let stream = consumer
            .messages()
            .await
            .map_err(|e| Error::StreamCreationFailed(e.to_string()))?;

        Ok(MessageStream(stream))
    }
}
