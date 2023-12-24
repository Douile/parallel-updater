use std::backtrace::{Backtrace, BacktraceStatus};

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    Library,
    MissingConfigFile,
    ConfigParseError,
}

pub struct Error {
    kind: ErrorKind,
    source: Box<dyn std::error::Error + 'static>,
    backtrace: Backtrace,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}( {} )", self.kind, self.source)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}( {:?} )", self.kind, self.source)?;
        if self.backtrace.status() == BacktraceStatus::Captured {
            writeln!(f)?;
            writeln!(f, "{:#?}", self.backtrace)?;
        }

        Ok(())
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
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Error {
        ErrorKind::MissingConfigFile.context(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Error {
        ErrorKind::ConfigParseError.context(value)
    }
}

impl From<parallel_update::error::Error> for Error {
    fn from(value: parallel_update::error::Error) -> Error {
        ErrorKind::Library.context(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
