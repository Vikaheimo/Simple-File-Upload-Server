use axum::{http::StatusCode, response::IntoResponse};
use log::warn;
use thiserror::Error;

pub type ApplicationResult<T> = Result<T, ApplicationError>;

#[derive(Debug, Error)]
pub struct ApplicationError {
    #[source]
    pub source: anyhow::Error,
    pub kind: ErrorKind,
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.kind)
    }
}

#[derive(Debug, Clone, strum_macros::Display)]
pub enum ErrorKind {
    #[strum(to_string = "File '{0}' not found on the server!")]
    FileNotFound(String),
    #[strum(to_string = "Invalid filename!")]
    InvalidFilename,
    #[strum(to_string = "File upload failed!")]
    FileUpload,
    #[strum(to_string = "Internal server error!")]
    Internal,
}

impl ApplicationError {
    pub fn from_io_with_path(err: std::io::Error, path: impl Into<String>) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self {
                kind: ErrorKind::FileNotFound(path.into()),
                source: err.into(),
            },
            _ => Self {
                source: err.into(),
                kind: ErrorKind::Internal,
            },
        }
    }
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self.kind {
            ErrorKind::FileNotFound(_) => StatusCode::NOT_FOUND,
            ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::FileUpload | ErrorKind::InvalidFilename => StatusCode::BAD_REQUEST,
        };

        warn!("{}", self.kind);
        if crate::ENVIRONMENT.verbose {
            warn!("{}", self.source.backtrace())
        }

        (status_code, self.kind.to_string()).into_response()
    }
}

impl From<axum::extract::multipart::MultipartError> for ApplicationError {
    fn from(value: axum::extract::multipart::MultipartError) -> Self {
        Self {
            source: value.into(),
            kind: ErrorKind::FileUpload,
        }
    }
}

impl From<askama::Error> for ApplicationError {
    fn from(value: askama::Error) -> Self {
        Self {
            source: value.into(),
            kind: ErrorKind::Internal,
        }
    }
}

impl From<std::io::Error> for ApplicationError {
    fn from(value: std::io::Error) -> Self {
        Self {
            source: value.into(),
            kind: ErrorKind::Internal,
        }
    }
}
