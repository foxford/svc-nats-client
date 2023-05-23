use async_nats::jetstream::consumer::pull::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub use crate::{
    client::{Client, PublishError, SubscribeError, TermMessageError},
    config::{Config, ConsumerConfig},
    event::Event,
    headers::Headers,
    subject::Subject,
};
pub use async_nats::jetstream::{
    consumer::{push::Messages, AckPolicy, DeliverPolicy},
    AckKind, Message,
};

pub mod consumer;
pub mod event;

mod client;
mod config;
mod headers;
mod subject;

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

    async fn subscribe_durable(&self) -> Result<MessageStream, SubscribeError>;

    async fn subscribe_ephemeral(
        &self,
        subject: Subject,
        deliver_policy: DeliverPolicy,
        ack_policy: AckPolicy,
    ) -> Result<Messages, SubscribeError>;

    async fn terminate(&self, message: Message) -> Result<(), TermMessageError>;
}
