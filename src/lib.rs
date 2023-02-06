use anyhow::Result;
use async_nats::jetstream::consumer::pull::Stream;
use async_trait::async_trait;
use futures::stream::Take;

pub use crate::client::new;
pub use crate::headers::HeaderMap;

pub mod client;
pub mod headers;

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
