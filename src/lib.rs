use anyhow::Result;
use async_nats::jetstream::consumer::pull::Stream;
use async_trait::async_trait;
use futures::stream::{Next, Take};
use futures::StreamExt;

pub use crate::client::new;
pub use crate::headers::{HeaderMap, HeaderMapBuilder};

pub mod client;
pub mod headers;

pub struct MessageStream(Take<Stream>);

pub use async_nats::jetstream::Message;

impl MessageStream {
    pub fn next(&mut self) -> Next<'_, Take<Stream>> {
        self.0.next()
    }
}

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
    ) -> Result<MessageStream>;
}
