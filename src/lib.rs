use async_nats::jetstream::consumer::pull::Stream;
use async_trait::async_trait;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub use crate::{
    client::{new, PublishError, SubscribeError},
    headers::HeaderMap,
    subject::Subject,
};
pub use async_nats::jetstream::AckKind;
pub use async_nats::jetstream::Message;

mod client;
mod headers;
mod subject;

pub struct MessageStream(Stream);

impl futures::Stream for MessageStream {
    type Item = Result<Message, async_nats::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}

#[async_trait]
pub trait NatsClient: Send + Sync {
    async fn publish(
        &self,
        subject: Subject,
        payload: Vec<u8>,
        headers: HeaderMap,
    ) -> Result<(), PublishError>;

    async fn subscribe(
        &self,
        stream: &str,
        consumer: &str,
    ) -> Result<MessageStream, SubscribeError>;
}
