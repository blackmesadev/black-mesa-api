use thiserror::Error;

#[derive(Error, Debug)]
enum ErrorRepr {
    #[error("jwt error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("mongodb error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("actix error: {0}")]
    HyperError(#[from] hyper::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("unknown error: {0}")]
    Unknown(String),
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct Error(#[from] ErrorRepr);

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self {
            0: ErrorRepr::JwtError(e),
        }
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        Self {
            0: ErrorRepr::Unknown(e.to_string()),
        }
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(e: mongodb::error::Error) -> Self {
        Self {
            0: ErrorRepr::MongoError(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self {
            0: ErrorRepr::IOError(e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self {
            0: ErrorRepr::SerdeError(e),
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(value: hyper::Error) -> Self {
        Self {
            0: ErrorRepr::HyperError(value),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self {
            0: ErrorRepr::ReqwestError(value),
        }
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Self {
            0: ErrorRepr::Unknown(e),
        }
    }
}
