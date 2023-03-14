use crate::{event_id::EventId, headers::Headers, subject::Subject};
use svc_agent::AgentId;

#[derive(Debug)]
pub struct Event {
    pub subject: Subject,
    pub payload: Vec<u8>,
    pub headers: Headers,
}

impl Event {
    pub fn new(subject: Subject, payload: Vec<u8>, event_id: EventId, agent_id: AgentId) -> Self {
        let headers = Headers::new(event_id, agent_id);

        Self {
            subject,
            payload,
            headers,
        }
    }
}
