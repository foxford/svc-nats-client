use crate::event_id::{EventId, EventIdError};
use std::str::FromStr;
use svc_agent::AgentId;

const SENDER_AGENT_ID: &str = "Sender-Agent-Id";
const ENTITY_EVENT_SEQUENCE_ID: &str = "Entity-Event-Sequence-Id";

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error("failed to get `{0}`")]
    InvalidHeader(String),
    #[error("failed to parse event_id")]
    InvalidEventId(#[from] EventIdError),
    #[error("failed to parse sender_id: `{0}`")]
    SenderIdParseFailed(String),
}

#[derive(Debug, Clone)]
pub struct Headers {
    event_id: EventId,
    sender_id: AgentId,
}

impl Headers {
    pub(crate) fn new(event_id: EventId, sender_id: AgentId) -> Self {
        Self {
            event_id,
            sender_id,
        }
    }

    pub fn sender_id(&self) -> &AgentId {
        &self.sender_id
    }

    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }
}

impl From<Headers> for async_nats::HeaderMap {
    fn from(value: Headers) -> Self {
        let mut headers = async_nats::HeaderMap::new();

        headers.insert(
            async_nats::header::NATS_MESSAGE_ID,
            value.event_id.to_string().as_str(),
        );
        headers.insert(
            ENTITY_EVENT_SEQUENCE_ID,
            value.event_id.sequence_id().to_string().as_str(),
        );
        headers.insert(SENDER_AGENT_ID, value.sender_id.to_string().as_str());

        headers
    }
}

impl TryFrom<async_nats::HeaderMap> for Headers {
    type Error = HeaderError;

    fn try_from(value: async_nats::HeaderMap) -> Result<Self, Self::Error> {
        let event_id =
            value
                .get(async_nats::header::NATS_MESSAGE_ID)
                .ok_or(HeaderError::InvalidHeader(
                    async_nats::header::NATS_MESSAGE_ID.to_string(),
                ))?;
        let event_id = event_id.try_into().map_err(HeaderError::InvalidEventId)?;

        let sender_id = value
            .get(async_nats::header::NATS_MESSAGE_ID)
            .ok_or(HeaderError::InvalidHeader(SENDER_AGENT_ID.to_string()))?
            .as_str();
        let sender_id = AgentId::from_str(sender_id)
            .map_err(|e| HeaderError::SenderIdParseFailed(e.to_string()))?;

        Ok(Self {
            event_id,
            sender_id,
        })
    }
}
