use std::sync::{Arc, RwLock};

use async_nats::jetstream::{
    consumer::{push::Messages, AckPolicy, DeliverPolicy},
    Message,
};

use crate::{
    event::Event, MessageStream, NatsClient, PublishError, Subject, SubscribeError,
    TermMessageError,
};

pub struct TestNatsClient {
    publish_requests: Arc<RwLock<Vec<Event>>>,
    terminate_requests: Arc<RwLock<Vec<Message>>>,
}

impl Default for TestNatsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TestNatsClient {
    pub fn new() -> Self {
        Self {
            publish_requests: Arc::new(RwLock::new(vec![])),
            terminate_requests: Arc::new(RwLock::new(vec![])),
        }
    }

    pub fn get_publish_requests(&self) -> std::sync::RwLockReadGuard<'_, Vec<Event>> {
        self.publish_requests
            .read()
            .expect("failed to get read lock on publish reqs")
    }

    pub fn get_terminate_requests(&self) -> std::sync::RwLockReadGuard<'_, Vec<Message>> {
        self.terminate_requests
            .read()
            .expect("failed to get read lock on terminate reqs")
    }
}

#[async_trait::async_trait]
impl NatsClient for TestNatsClient {
    async fn publish(&self, event: &Event) -> Result<(), PublishError> {
        let mut reqs = self
            .publish_requests
            .write()
            .expect("failed to get write lock on publish reqs");

        reqs.push(event.clone());

        Ok(())
    }

    async fn subscribe_durable(&self) -> Result<MessageStream, SubscribeError> {
        unimplemented!("this is test client")
    }

    async fn subscribe_ephemeral(
        &self,
        _subject: Subject,
        _deliver_policy: DeliverPolicy,
        _ack_policy: AckPolicy,
    ) -> Result<Messages, SubscribeError> {
        unimplemented!("this is test client")
    }

    async fn terminate(&self, message: &Message) -> Result<(), TermMessageError> {
        let mut reqs = self
            .terminate_requests
            .write()
            .expect("failed to get write lock on terminate reqs");

        reqs.push(message.clone());

        Ok(())
    }
}
