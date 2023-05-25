use std::str::FromStr;
use svc_agent::AgentId;
use svc_events::EventId;

const SENDER_ID: &str = "Sender-Agent-Id";
const ENTITY_EVENT_SEQUENCE_ID: &str = "Entity-Event-Sequence-Id";
const ENTITY_EVENT_TYPE: &str = "Entity-Event-Type";
const ENTITY_EVENT_OPERATION: &str = "Entity-Event-Operation";
const IS_INTERNAL: &str = "Is-Internal";
const RECEIVER_ID: &str = "Receiver-Agent-Id";

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error("failed to get `{0}`")]
    InvalidHeader(String),
    #[error("failed to parse entity_sequence_id")]
    InvalidSequenceId(#[from] std::num::ParseIntError),
    #[error(transparent)]
    AgentIdParseFailed(#[from] svc_agent::Error),
    #[error("failed to parse is_internal")]
    InvalidIsInternal(#[from] std::str::ParseBoolError),
}

#[derive(Debug, Clone)]
pub struct Headers {
    event_id: EventId,
    sender_id: AgentId,
    is_internal: bool,
    receiver_id: Option<AgentId>,
    is_deduplication_enabled: bool,
}

impl Headers {
    pub fn sender_id(&self) -> &AgentId {
        &self.sender_id
    }

    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }

    pub fn is_internal(&self) -> bool {
        self.is_internal
    }

    pub fn receiver_id(&self) -> Option<&AgentId> {
        self.receiver_id.as_ref()
    }
}

pub(crate) struct Builder {
    event_id: EventId,
    sender_id: AgentId,
    is_internal: bool,
    receiver_id: Option<AgentId>,
    is_deduplication_enabled: bool,
}

impl Builder {
    pub(crate) fn new(event_id: EventId, sender_id: AgentId) -> Self {
        Self {
            event_id,
            sender_id,
            is_internal: true,
            receiver_id: None,
            is_deduplication_enabled: true,
        }
    }

    pub(crate) fn internal(self, is_internal: bool) -> Self {
        Self {
            is_internal,
            ..self
        }
    }

    pub(crate) fn receiver_id(self, receiver_id: AgentId) -> Self {
        Self {
            receiver_id: Some(receiver_id),
            ..self
        }
    }

    pub(crate) fn enable_deduplication(self, is_deduplication_enabled: bool) -> Self {
        Self {
            is_deduplication_enabled,
            ..self
        }
    }

    pub(crate) fn build(self) -> Headers {
        Headers {
            event_id: self.event_id,
            sender_id: self.sender_id,
            is_internal: self.is_internal,
            receiver_id: self.receiver_id,
            is_deduplication_enabled: self.is_deduplication_enabled,
        }
    }
}

impl From<Headers> for async_nats::HeaderMap {
    fn from(value: Headers) -> Self {
        let mut headers = async_nats::HeaderMap::new();

        let event_id = value.event_id();

        if value.is_deduplication_enabled {
            headers.insert(
                async_nats::header::NATS_MESSAGE_ID,
                event_id.to_string().as_str(),
            );
        }

        headers.insert(ENTITY_EVENT_TYPE, event_id.entity_type());
        headers.insert(
            ENTITY_EVENT_SEQUENCE_ID,
            event_id.sequence_id().to_string().as_str(),
        );
        headers.insert(SENDER_ID, value.sender_id().to_string().as_str());
        headers.insert(IS_INTERNAL, value.is_internal().to_string().as_str());

        if let Some(receiver_id) = value.receiver_id() {
            headers.insert(RECEIVER_ID, receiver_id.to_string().as_str());
        }

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

        let operation = value
            .get(ENTITY_EVENT_OPERATION)
            .ok_or(HeaderError::InvalidHeader(
                ENTITY_EVENT_OPERATION.to_string(),
            ))?
            .to_string();

        let sequence_id = value
            .get(ENTITY_EVENT_SEQUENCE_ID)
            .ok_or(HeaderError::InvalidHeader(
                ENTITY_EVENT_SEQUENCE_ID.to_string(),
            ))?
            .as_str()
            .parse::<i64>()?;

        let event_id = (entity_type, operation, sequence_id).into();

        let sender_id = value
            .get(SENDER_ID)
            .ok_or(HeaderError::InvalidHeader(SENDER_ID.to_string()))?
            .as_str();
        let sender_id = AgentId::from_str(sender_id).map_err(HeaderError::AgentIdParseFailed)?;

        let is_internal = value
            .get(IS_INTERNAL)
            .ok_or(HeaderError::InvalidHeader(IS_INTERNAL.to_string()))?
            .as_str()
            .parse::<bool>()?;

        let receiver_id = value
            .get(RECEIVER_ID)
            .map(|h| AgentId::from_str(h.as_str()).map_err(HeaderError::AgentIdParseFailed))
            .transpose()?;

        let is_deduplication_enabled = value.get(async_nats::header::NATS_MESSAGE_ID).is_some();

        Ok(Self {
            event_id,
            sender_id,
            is_internal,
            receiver_id,
            is_deduplication_enabled,
        })
    }
}
