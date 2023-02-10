const SENDER_AGENT_ID: &str = "Sender-Agent-Id";

#[derive(Default)]
pub struct HeaderMap {
    inner: async_nats::HeaderMap,
}

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error("failed to get `{0}`")]
    HeaderNotFound(String),
}

impl HeaderMap {
    pub fn new(message_id: &str, agent_id: &str) -> Self {
        let mut inner = async_nats::HeaderMap::default();

        inner.insert(async_nats::header::NATS_MESSAGE_ID, message_id);
        inner.insert(SENDER_AGENT_ID, agent_id);

        Self { inner }
    }

    pub fn sender_agent_id(&self) -> Result<&str, HeaderError> {
        self.inner
            .get(SENDER_AGENT_ID)
            .map(|v| v.into())
            .ok_or(HeaderError::HeaderNotFound(SENDER_AGENT_ID.to_string()))
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
