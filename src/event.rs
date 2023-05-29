use crate::{
    event_id::EventId,
    headers::{Builder as HeadersBuilder, Headers},
    subject::Subject,
};
use svc_agent::AgentId;

#[derive(Debug)]
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
    deduplication: bool,
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
            deduplication: true,
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

    pub fn deduplication(self, deduplication: bool) -> Self {
        Self {
            deduplication,
            ..self
        }
    }

    pub fn build(self) -> Event {
        let mut builder = HeadersBuilder::new(self.event_id, self.sender_id)
            .internal(self.is_internal)
            .deduplication(self.deduplication);

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
