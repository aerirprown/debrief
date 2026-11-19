use std::path::PathBuf;

/// Errors from journal read/write.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("json error at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid device id file at {path}")]
    InvalidDeviceId { path: PathBuf },
    #[error("file already exists (append-only): {path}")]
    AlreadyExists { path: PathBuf },
    #[error("missing field {field} in {path}")]
    MissingField { path: PathBuf, field: &'static str },
    #[error("not found: {what}")]
    NotFound { what: String },
    #[error("note has an open fork: {note_id}")]
    OpenFork { note_id: String },
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Error {
    Error::Io {
        path: path.into(),
        source,
    }
}

pub(crate) fn json(path: impl Into<PathBuf>, source: serde_json::Error) -> Error {
    Error::Json {
        path: path.into(),
        source,
    }
}
