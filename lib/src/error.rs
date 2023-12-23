#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    InvalidUpdater,
    InvalidConfig,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    source: Box<dyn std::error::Error + 'static>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}( {} )", self.kind, self.source)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl ErrorKind {
    pub fn context<E: Into<Box<dyn std::error::Error + 'static>>>(self, context: E) -> Error {
        Error {
            kind: self,
            source: context.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
