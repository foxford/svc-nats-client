use crate::{headers::HeaderMap, MessageStream, NatsClient};
use anyhow::{anyhow, Result};
use async_nats::{
    jetstream::{consumer::PullConsumer, Context},
    Event,
};
use async_trait::async_trait;
use tracing::{error, warn};

#[derive(Clone)]
pub struct Client {
    pub jetstream: Context,
}

pub async fn new(url: &str, creds: &str) -> Result<Client> {
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
    ) -> Result<()> {
        self.jetstream
            .publish_with_headers(subject, headers.unwrap_or_default().into(), payload.into())
            .await
            .map_err(|e| anyhow!("failed to publish message through nats: {}", e))?
            .await
            .map_err(|e| anyhow!("failed to ack message: {}", e))?;

        Ok(())
    }

    async fn subscribe(&self, stream: &str, consumer: &str) -> Result<MessageStream> {
        let stream = self
            .jetstream
            .get_stream(stream)
            .await
            .map_err(|e| anyhow!("failed to get stream: {}", e))?;

        let consumer: PullConsumer = stream
            .get_consumer(consumer)
            .await
            .map_err(|e| anyhow!("failed to pull consumer: {}", e))?;

        let stream = consumer
            .messages()
            .await
            .map_err(|e| anyhow!("failed to create stream of messages: {}", e))?;

        Ok(MessageStream(stream))
    }
}
