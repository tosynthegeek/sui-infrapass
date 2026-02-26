use crate::adapters::error::ProxyError;

#[derive(Debug)]
pub enum InfrapassError {
    DatabaseError(String),
    AdapterError(String),
    EventProcessingError(String),
    ValidationError(String),
    Other(String),
    ProxyError(ProxyError),
    RedisError(redis::RedisError),
    SerdeError(serde_json::Error),
}

impl std::fmt::Display for InfrapassError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfrapassError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            InfrapassError::AdapterError(msg) => write!(f, "Adapter error: {}", msg),
            InfrapassError::EventProcessingError(msg) => {
                write!(f, "Event processing error: {}", msg)
            }
            InfrapassError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            InfrapassError::Other(msg) => write!(f, "Other error: {}", msg),
            InfrapassError::ProxyError(err) => write!(f, "Proxy error: {}", err),
            InfrapassError::RedisError(err) => write!(f, "Redis error: {}", err),
            InfrapassError::SerdeError(err) => write!(f, "Serde error: {}", err),
        }
    }
}

impl std::error::Error for InfrapassError {}

impl From<ProxyError> for InfrapassError {
    fn from(err: ProxyError) -> Self {
        InfrapassError::ProxyError(err)
    }
}

impl From<redis::RedisError> for InfrapassError {
    fn from(err: redis::RedisError) -> Self {
        InfrapassError::RedisError(err)
    }
}

impl From<serde_json::Error> for InfrapassError {
    fn from(err: serde_json::Error) -> Self {
        InfrapassError::SerdeError(err)
    }
}
