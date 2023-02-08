const SENDER_AGENT_ID: &str = "Sender-Agent-Id";

#[derive(Default)]
pub struct HeaderMap {
    inner: async_nats::HeaderMap,
}

#[derive(Default)]
pub struct HeaderMapBuilder {
    inner: async_nats::HeaderMap,
}

impl HeaderMapBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn message_id(mut self, message_id: &str) -> Self {
        self.inner
            .insert(async_nats::header::NATS_MESSAGE_ID, message_id);
        self
    }

    pub fn sender_agent_id(mut self, agent_id: &str) -> Self {
        self.inner.insert(SENDER_AGENT_ID, agent_id);
        self
    }

    pub fn build(self) -> HeaderMap {
        HeaderMap { inner: self.inner }
    }
}

impl HeaderMap {
    pub fn message_id(&self) -> Option<&str> {
        self.inner
            .get(async_nats::header::NATS_MESSAGE_ID)
            .map(|v| v.into())
    }

    pub fn sender_agent_id(&self) -> Option<&str> {
        self.inner.get(SENDER_AGENT_ID).map(|v| v.into())
    }
}

impl From<async_nats::HeaderMap> for HeaderMap {
    fn from(value: async_nats::HeaderMap) -> Self {
        Self { inner: value }
    }
}

impl From<HeaderMap> for async_nats::HeaderMap {
    fn from(value: HeaderMap) -> Self {
        value.inner
    }
}
