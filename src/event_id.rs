// fixme: move to the svc-outbox crate
#[derive(Debug, Clone)]
pub struct EventId {
    entity_type: String,
    sequence_id: i64,
}

impl EventId {
    pub fn entity_type(&self) -> &str {
        &self.entity_type
    }

    pub fn sequence_id(&self) -> i64 {
        self.sequence_id
    }
}

impl From<(String, i64)> for EventId {
    fn from((entity_type, sequence_id): (String, i64)) -> Self {
        Self {
            entity_type,
            sequence_id,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventIdError {
    #[error("failed to get entity_type from the string")]
    InvalidEntityType,
    #[error("failed to get sequence_id from the string")]
    InvalidSequenceId,
    #[error(transparent)]
    SequenceIdParseFailed(#[from] std::num::ParseIntError),
}

impl<'a> TryFrom<&'a async_nats::HeaderValue> for EventId {
    type Error = EventIdError;

    fn try_from(value: &'a async_nats::HeaderValue) -> Result<Self, Self::Error> {
        std::str::FromStr::from_str(value.as_str())
    }
}

impl std::str::FromStr for EventId {
    type Err = EventIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut event_id = s.split('_').fuse();
        let entity_type = event_id
            .next()
            .ok_or(EventIdError::InvalidEntityType)?
            .to_string();
        let sequence_id = event_id
            .next()
            .ok_or(EventIdError::InvalidSequenceId)?
            .parse::<i64>()
            .map_err(EventIdError::SequenceIdParseFailed)?;

        Ok(Self {
            entity_type,
            sequence_id,
        })
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.entity_type, self.sequence_id)
    }
}
