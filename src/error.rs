use thiserror::Error;

/// Main error type for the Modal SDK
#[derive(Error, Debug)]
pub enum Error {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<serde_cbor::Error> for Error {
    fn from(err: serde_cbor::Error) -> Self {
        Error::Serialization(format!("CBOR error: {}", err))
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Error::Connection(format!("Transport error: {}", err))
    }
}

