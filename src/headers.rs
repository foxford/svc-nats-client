const SENDER_AGENT_ID: &str = "Sender-Agent-Id";

#[derive(Default)]
pub struct HeaderMap {
    inner: async_nats::HeaderMap,
}

impl HeaderMap {
    pub fn new() -> Self {
        Self {
            inner: async_nats::HeaderMap::new(),
        }
    }

    pub fn nats_message_id(mut self, message_id: &str) -> Self {
        self.inner
            .insert(async_nats::header::NATS_MESSAGE_ID, message_id);
        self
    }

    pub fn sender_agent_id(mut self, agent_id: &str) -> Self {
        self.inner.insert(SENDER_AGENT_ID, agent_id);
        self
    }

    pub fn get(self) -> async_nats::HeaderMap {
        self.inner
    }
}
