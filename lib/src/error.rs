use std::backtrace::{Backtrace, BacktraceStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidUpdater,
    InvalidConfig,
    CommandSpawn,
    CommandOutput,
    IOError,
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

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! context {
    ($kind: expr, $($arg:tt)*) => {
        $kind.context(format!($($arg)*))
    };
}
pub(crate) use context;

macro_rules! bail {
    ($kind: expr, $($arg:tt)*) => {
        return Err(crate::error::context!($kind, $($arg)*))
    }
}
pub(crate) use bail;
