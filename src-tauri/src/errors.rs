use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
#[allow(dead_code)] // Acknowledging some variants/methods might be unused currently
pub enum DatomicError {
    #[error("JNI error: {0}")]
    JniError(#[from] jni::errors::Error),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
    
    #[error("Schema error: {0}")]
    SchemaError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Database not found: {0}")]
    DatabaseNotFound(String),
    
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    
    #[error("Timeout error: operation timed out after {timeout_ms}ms")]
    TimeoutError { timeout_ms: u64 },
    
    #[error("Retry limit exceeded: {attempts} attempts failed")]
    RetryLimitExceeded { attempts: u32 },
    
    #[error("Invalid entity ID: {0}")]
    InvalidEntityId(String),
    
    #[error("Invalid transaction data: {0}")]
    InvalidTransactionData(String),
    
    #[error("Java class not found: {0}")]
    JavaClassNotFound(String),
    
    #[error("Java method not found: {0}")]
    JavaMethodNotFound(String),
    
    #[error("JVM initialization failed: {0}")]
    JvmInitializationFailed(String),
    
    #[error("EDN parsing error: {0}")]
    EdnParsingError(String),
    
    #[error("Type conversion error: {0}")]
    TypeConversionError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[allow(dead_code)] // Acknowledging some constructor methods might be unused currently
impl DatomicError {
    pub fn connection_error<T: Into<String>>(msg: T) -> Self {
        DatomicError::ConnectionError(msg.into())
    }
    
    pub fn transaction_error<T: Into<String>>(msg: T) -> Self {
        DatomicError::TransactionError(msg.into())
    }
    
    pub fn query_error<T: Into<String>>(msg: T) -> Self {
        DatomicError::QueryError(msg.into())
    }
    
    pub fn schema_error<T: Into<String>>(msg: T) -> Self {
        DatomicError::SchemaError(msg.into())
    }
    
    pub fn serialization_error<T: Into<String>>(msg: T) -> Self {
        DatomicError::SerializationError(msg.into())
    }
    
    pub fn config_error<T: Into<String>>(msg: T) -> Self {
        DatomicError::ConfigError(msg.into())
    }
    
    pub fn database_not_found<T: Into<String>>(msg: T) -> Self {
        DatomicError::DatabaseNotFound(msg.into())
    }
    
    pub fn entity_not_found<T: Into<String>>(msg: T) -> Self {
        DatomicError::EntityNotFound(msg.into())
    }
    
    pub fn timeout_error(timeout_ms: u64) -> Self {
        DatomicError::TimeoutError { timeout_ms }
    }
    
    pub fn retry_limit_exceeded(attempts: u32) -> Self {
        DatomicError::RetryLimitExceeded { attempts }
    }
    
    pub fn invalid_entity_id<T: Into<String>>(msg: T) -> Self {
        DatomicError::InvalidEntityId(msg.into())
    }
    
    pub fn invalid_transaction_data<T: Into<String>>(msg: T) -> Self {
        DatomicError::InvalidTransactionData(msg.into())
    }
    
    pub fn java_class_not_found<T: Into<String>>(msg: T) -> Self {
        DatomicError::JavaClassNotFound(msg.into())
    }
    
    pub fn java_method_not_found<T: Into<String>>(msg: T) -> Self {
        DatomicError::JavaMethodNotFound(msg.into())
    }
    
    pub fn jvm_initialization_failed<T: Into<String>>(msg: T) -> Self {
        DatomicError::JvmInitializationFailed(msg.into())
    }
    
    pub fn edn_parsing_error<T: Into<String>>(msg: T) -> Self {
        DatomicError::EdnParsingError(msg.into())
    }
    
    pub fn type_conversion_error<T: Into<String>>(msg: T) -> Self {
        DatomicError::TypeConversionError(msg.into())
    }
    
    pub fn internal_error<T: Into<String>>(msg: T) -> Self {
        DatomicError::InternalError(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, DatomicError>;

#[macro_export]
macro_rules! datomic_error {
    ($kind:ident, $msg:expr) => {
        DatomicError::$kind($msg.to_string())
    };
    ($kind:ident, $fmt:expr, $($arg:tt)*) => {
        DatomicError::$kind(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! bail {
    ($kind:ident, $msg:expr) => {
        return Err(datomic_error!($kind, $msg))
    };
    ($kind:ident, $fmt:expr, $($arg:tt)*) => {
        return Err(datomic_error!($kind, $fmt, $($arg)*))
    };
}

/// Retry configuration for database operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Execute a fallible operation with retry logic
pub async fn with_retry<F, T, E>(
    mut operation: F, // Added mut here
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T>
where
    F: FnMut() -> std::result::Result<T, E> + Send + Sync, // Changed Fn to FnMut
    E: fmt::Display + fmt::Debug + Send + Sync,
{
    let mut delay = config.initial_delay_ms;
    let mut last_error: Option<E> = None;
    
    for attempt in 1..=config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::warn!(
                    "Operation '{}' failed on attempt {} of {}: {}",
                    operation_name,
                    attempt,
                    config.max_attempts,
                    e
                );
                
                last_error = Some(e);
                
                if attempt < config.max_attempts {
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    delay = (delay as f64 * config.backoff_multiplier) as u64;
                    delay = delay.min(config.max_delay_ms);
                }
            }
        }
    }
    
    if let Some(_e) = last_error { // Prefixed e with underscore
        Err(DatomicError::RetryLimitExceeded {
            attempts: config.max_attempts,
        })
    } else {
        Err(DatomicError::InternalError(
            "Retry loop completed without result".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = DatomicError::connection_error("Connection failed");
        assert_eq!(err.to_string(), "Connection error: Connection failed");
        
        let err = DatomicError::timeout_error(5000);
        assert_eq!(err.to_string(), "Timeout error: operation timed out after 5000ms");
    }
    
    #[tokio::test]
    async fn test_retry_success() {
        let config = RetryConfig::default();
        let mut attempt_count = 0;
        
        let result = with_retry(
            || {
                attempt_count += 1;
                if attempt_count < 3 {
                    Err("Temporary failure")
                } else {
                    Ok(42)
                }
            },
            &config,
            "test_operation",
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count, 3);
    }
    
    #[tokio::test]
    async fn test_retry_failure() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 1,
            max_delay_ms: 10,
            backoff_multiplier: 2.0,
        };
        
        let result: Result<i32> = with_retry( // Added type annotation Result<i32> which implies Result<i32, DatomicError>
            || Err("Always fails"),
            &config,
            "test_operation",
        ).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            DatomicError::RetryLimitExceeded { attempts } => {
                assert_eq!(attempts, 2);
            }
            _ => panic!("Expected RetryLimitExceeded error"),
        }
    }
}
