use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ClutterError>;

#[derive(Debug, Error)]
pub enum ClutterError {
    #[error("usage error: {0}")]
    Usage(String),

    #[error("invalid artifact: {0}")]
    InvalidArtifact(String),

    #[error("unsupported input: {0}")]
    Unsupported(String),

    #[error("analysis failed: {0}")]
    Analysis(String),

    #[error("output path already exists: {0}")]
    OutputExists(PathBuf),

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("ELF error: {0}")]
    Object(#[from] object::Error),
}

impl ClutterError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => 2,
            Self::InvalidArtifact(_) => 3,
            Self::Unsupported(_) => 4,
            Self::Analysis(_) => 5,
            Self::OutputExists(_) | Self::Io { .. } => 6,
            Self::Zip(_) | Self::Json(_) | Self::Object(_) => 5,
        }
    }
}

pub trait IoContext<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> IoContext<T> for std::result::Result<T, io::Error> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T> {
        self.map_err(|source| ClutterError::Io {
            path: path.into(),
            source,
        })
    }
}
