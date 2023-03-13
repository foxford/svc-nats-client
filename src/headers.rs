use crate::event_id::EventId;
use std::str::FromStr;
use svc_agent::AgentId;

const SENDER_AGENT_ID: &str = "Sender-Agent-Id";
const ENTITY_EVENT_SEQUENCE_ID: &str = "Entity-Event-Sequence-Id";
const ENTITY_EVENT_TYPE: &str = "Entity-Event-Type";

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error("failed to get `{0}`")]
    InvalidHeader(String),
    #[error("failed to parse entity sequence id")]
    InvalidSequenceId(#[from] std::num::ParseIntError),
    #[error(transparent)]
    SenderIdParseFailed(#[from] svc_agent::Error),
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

        headers.insert(ENTITY_EVENT_TYPE, value.event_id.entity_type());
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
        let entity_type = value
            .get(ENTITY_EVENT_TYPE)
            .ok_or(HeaderError::InvalidHeader(ENTITY_EVENT_TYPE.to_string()))?
            .to_string();

        let sequence_id = value
            .get(ENTITY_EVENT_SEQUENCE_ID)
            .ok_or(HeaderError::InvalidHeader(
                ENTITY_EVENT_SEQUENCE_ID.to_string(),
            ))?
            .as_str()
            .parse::<i64>()?;

        let event_id = (entity_type, sequence_id).into();

        let sender_id = value
            .get(SENDER_AGENT_ID)
            .ok_or(HeaderError::InvalidHeader(SENDER_AGENT_ID.to_string()))?
            .as_str();
        let sender_id = AgentId::from_str(sender_id).map_err(HeaderError::SenderIdParseFailed)?;

        Ok(Self {
            event_id,
            sender_id,
        })
    }
}
