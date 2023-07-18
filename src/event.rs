use crate::{
    headers::{Builder as HeadersBuilder, Headers},
    subject::Subject,
};
use svc_agent::AgentId;
use svc_events::EventId;

#[derive(Debug, Clone)]
pub struct Event {
    pub subject: Subject,
    pub payload: Vec<u8>,
    pub headers: Headers,
}

impl Event {
    pub fn subject(&self) -> &Subject {
        &self.subject
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }
}

pub struct Builder {
    subject: Subject,
    payload: Vec<u8>,
    event_id: EventId,
    sender_id: AgentId,
    is_internal: bool,
    receiver_id: Option<AgentId>,
    is_deduplication_enabled: bool,
}

impl Builder {
    pub fn new(subject: Subject, payload: Vec<u8>, event_id: EventId, sender_id: AgentId) -> Self {
        Self {
            subject,
            payload,
            event_id,
            sender_id,
            is_internal: true,
            receiver_id: None,
            is_deduplication_enabled: true,
        }
    }

    pub fn internal(self, is_internal: bool) -> Self {
        Self {
            is_internal,
            ..self
        }
    }

    pub fn receiver_id(self, receiver_id: AgentId) -> Self {
        Self {
            receiver_id: Some(receiver_id),
            ..self
        }
    }

    pub fn disable_deduplication(self) -> Self {
        Self {
            is_deduplication_enabled: false,
            ..self
        }
    }

    pub fn build(self) -> Event {
        let mut builder = HeadersBuilder::new(self.event_id, self.sender_id)
            .internal(self.is_internal)
            .enable_deduplication(self.is_deduplication_enabled);

        if let Some(receiver_id) = self.receiver_id {
            builder = builder.receiver_id(receiver_id);
        }

        let headers = builder.build();

        Event {
            subject: self.subject,
            payload: self.payload,
            headers,
        }
    }
}
