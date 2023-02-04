use anyhow::{anyhow, Result};
use async_nats::{
    header::NATS_MESSAGE_ID,
    jetstream::{
        consumer::{pull::Stream, PullConsumer},
        Context,
    },
    Event,
};
use async_trait::async_trait;
use futures::{stream::Take, StreamExt};
use tracing::{error, warn};

#[async_trait]
pub trait NatsClient: Send + Sync {
    async fn publish(
        &self,
        subject: String,
        payload: Vec<u8>,
        headers: Option<HeaderMap>,
    ) -> Result<()>;
    async fn pull_messages(
        &self,
        stream_name: &str,
        consumer_name: &str,
        n: usize,
    ) -> Result<Take<Stream>>;
}

#[derive(Clone)]
pub struct Client {
    pub jetstream: Context,
}

#[derive(Default)]
pub struct HeaderMap {
    inner: async_nats::HeaderMap,
}

impl HeaderMap {
    pub fn nats_message_id(message_id: &str) -> Self {
        let mut headers = async_nats::HeaderMap::new();
        headers.insert(NATS_MESSAGE_ID, message_id);

        Self { inner: headers }
    }
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
                    error!(%error, "server error occurred");
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
            .publish_with_headers(subject, headers.unwrap_or_default().inner, payload.into())
            .await
            .map_err(|e| anyhow!("failed to publish message through nats: {}", e))?
            .await
            .map_err(|e| anyhow!("failed to ack message: {}", e))?;

        Ok(())
    }

    async fn pull_messages(
        &self,
        stream_name: &str,
        consumer_name: &str,
        n: usize,
    ) -> Result<Take<Stream>> {
        let stream = self
            .jetstream
            .get_stream(stream_name)
            .await
            .map_err(|e| anyhow!("failed to get stream: {}", e))?;

        let consumer: PullConsumer = stream
            .get_consumer(consumer_name)
            .await
            .map_err(|e| anyhow!("failed to pull consumer: {}", e))?;

        let messages = consumer
            .messages()
            .await
            .map_err(|e| anyhow!("failed to create stream of messages: {}", e))?
            .take(n);

        Ok(messages)
    }
}
