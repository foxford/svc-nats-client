use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) const TERMINATED_PREFIX: &str = "terminated";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subject {
    prefix: String,
    classroom_id: Uuid,
    entity_type: String,
}

impl Subject {
    pub fn new(prefix: String, classroom_id: Uuid, entity_type: String) -> Self {
        Self {
            prefix,
            classroom_id,
            entity_type,
        }
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn classroom_id(&self) -> Uuid {
        self.classroom_id
    }

    pub fn entity_type(&self) -> &str {
        &self.entity_type
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.prefix, self.classroom_id, self.entity_type
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubjectError {
    #[error("failed to get prefix from the subject")]
    PrefixNotFound,
    #[error("failed to get classroom_id from the subject")]
    ClassroomIdNotFound,
    #[error("failed to get entity_type from the subject")]
    EntityTypeNotFound,
    #[error(transparent)]
    ClassroomIdParseFailed(#[from] uuid::Error),
}

impl std::str::FromStr for Subject {
    type Err = SubjectError;

    fn from_str(subject: &str) -> Result<Self, Self::Err> {
        let mut subject = subject.split('.').fuse();
        let prefix = subject
            .next()
            .ok_or(SubjectError::PrefixNotFound)?
            .to_string();
        let classroom_id = subject.next().ok_or(SubjectError::ClassroomIdNotFound)?;
        let entity_type = subject
            .next()
            .ok_or(SubjectError::EntityTypeNotFound)?
            .to_string();

        let classroom_id =
            Uuid::parse_str(classroom_id).map_err(SubjectError::ClassroomIdParseFailed)?;

        Ok(Self {
            prefix,
            classroom_id,
            entity_type,
        })
    }
}
