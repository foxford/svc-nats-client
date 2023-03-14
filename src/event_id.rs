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

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.entity_type, self.sequence_id)
    }
}
