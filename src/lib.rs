use async_nats::jetstream::consumer::pull::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub use crate::{
    client::{new, PublishError, SubscribeError},
    config::Config,
    event::Event,
    event_id::EventId,
    headers::Headers,
    subject::Subject,
};
pub use async_nats::jetstream::{AckKind, Message};

mod client;
mod config;
mod event;
mod event_id;
mod headers;
mod subject;

pub mod error;

pub struct MessageStream(Stream);

impl futures::Stream for MessageStream {
    type Item = Result<Message, async_nats::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}

#[async_trait::async_trait]
pub trait NatsClient: Send + Sync {
    async fn publish(&self, event: &Event) -> Result<(), PublishError>;

    async fn subscribe(
        &self,
        stream: &str,
        consumer: &str,
    ) -> Result<MessageStream, SubscribeError>;
}
