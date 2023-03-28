use std::time::Duration;

#[derive(Clone, Copy)]
pub enum AckKind {
    Nak(Duration),
    Term,
}

pub struct Error {
    pub error: anyhow::Error,
    pub ack_kind: AckKind,
}

impl Error {
    pub fn new(error: anyhow::Error, ack_kind: AckKind) -> Self {
        Error { error, ack_kind }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.error)
    }
}

pub trait ErrorExt<T> {
    fn ack_with(self, kind: AckKind) -> Result<T, Error>;
}

impl<T, E: Into<anyhow::Error>> ErrorExt<T> for Result<T, E> {
    fn ack_with(self, kind: AckKind) -> Result<T, Error> {
        self.map_err(|source| Error::new(source.into(), kind))
    }
}
