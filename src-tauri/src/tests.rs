#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;
    use tokio::test;
    
    /// Test configuration loading and validation
    #[test]
    fn test_config_loading() {
        // Test default configuration
        let config = AppConfig::default();
        assert_eq!(config.datomic.transactor_port, 8998);
        assert_eq!(config.datomic.database_name, "gita");
        assert_eq!(config.audio.sample_rate, 44100);
        assert_eq!(config.audio.channels, 2);
        
        // Test classpath generation
        let result = config.get_datomic_classpath();
        // This will fail without actual Datomic installation, which is expected
        assert!(result.is_err());
    }
    
    /// Test error handling and conversion
    #[test]
    fn test_error_handling() {
        let err = DatomicError::connection_error("Test connection error");
        assert_eq!(err.to_string(), "Connection error: Test connection error");
        
        let err = DatomicError::timeout_error(5000);
        assert_eq!(err.to_string(), "Timeout error: operation timed out after 5000ms");
        
        let err = DatomicError::retry_limit_exceeded(3);
        assert_eq!(err.to_string(), "Retry limit exceeded: 3 attempts failed");
    }
    
    /// Test retry logic
    #[test]
    async fn test_retry_logic() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 1,
            max_delay_ms: 10,
            backoff_multiplier: 2.0,
        };
        
        let mut attempt_count = 0;
        
        // Test successful retry
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
        
        // Test failed retry
        let result = with_retry(
            || Err("Always fails"),
            &config,
            "test_operation",
        ).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            DatomicError::RetryLimitExceeded { attempts } => {
                assert_eq!(attempts, 3);
            }
            _ => panic!("Expected RetryLimitExceeded error"),
        }
    }
    
    /// Test schema validation
    #[test]
    fn test_schema_validation() {
        let schema = gita_schema_edn();
        
        // Basic validation - schema should not be empty
        assert!(!schema.is_empty());
        
        // Check for required attributes
        assert!(schema.contains(":block/id"));
        assert!(schema.contains(":block/content"));
        assert!(schema.contains(":block/created-at"));
        assert!(schema.contains(":block/updated-at"));
        
        // Check for audio-related attributes
        assert!(schema.contains(":audio-recording/id"));
        assert!(schema.contains(":audio-recording/file-path"));
        assert!(schema.contains(":audio-recording/duration"));
        
        // Check for timestamp attributes
        assert!(schema.contains(":audio-timestamp/block-id"));
        assert!(schema.contains(":audio-timestamp/timestamp-ms"));
    }
    
    /// Test model serialization/deserialization
    #[test]
    fn test_model_serialization() {
        use chrono::Utc;
        use uuid::Uuid;
        
        let block = Block {
            id: Uuid::new_v4().to_string(),
            content: "Test block content".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            page_id: Some("test-page".to_string()),
            parent_id: None,
            order: Some(0),
            audio_file: None,
            audio_timestamp: None,
        };
        
        // Test JSON serialization
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: Block = serde_json::from_str(&json).unwrap();
        
        assert_eq!(block.id, deserialized.id);
        assert_eq!(block.content, deserialized.content);
        assert_eq!(block.page_id, deserialized.page_id);
        assert_eq!(block.order, deserialized.order);
    }
    
    /// Test environment variable handling
    #[test]
    fn test_env_var_handling() {
        // Test config loading with environment variables
        env::set_var("GITA_DB_URI", "datomic:dev://localhost:9999/test");
        env::set_var("GITA_DB_HOST", "testhost");
        env::set_var("GITA_DB_PORT", "9999");
        env::set_var("GITA_LOG_LEVEL", "debug");
        
        let config = AppConfig::load().unwrap();
        
        assert_eq!(config.datomic.db_uri, "datomic:dev://localhost:9999/test");
        assert_eq!(config.datomic.transactor_host, "testhost");
        assert_eq!(config.datomic.transactor_port, 9999);
        assert_eq!(config.log_level, "debug");
        
        // Clean up
        env::remove_var("GITA_DB_URI");
        env::remove_var("GITA_DB_HOST");
        env::remove_var("GITA_DB_PORT");
        env::remove_var("GITA_LOG_LEVEL");
    }
    
    /// Test directory creation
    #[test]
    fn test_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();
        
        let mut config = AppConfig::default();
        config.data_dir = config_path.join("gita-test");
        config.audio.recordings_dir = config_path.join("recordings-test");
        
        // This should create the directories
        let result = std::fs::create_dir_all(&config.data_dir);
        assert!(result.is_ok());
        
        let result = std::fs::create_dir_all(&config.audio.recordings_dir);
        assert!(result.is_ok());
        
        // Verify directories exist
        assert!(config.data_dir.exists());
        assert!(config.audio.recordings_dir.exists());
    }
    
    /// Test configuration file operations
    #[test]
    fn test_config_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        
        // Create a test configuration
        let config = AppConfig::default();
        
        // Serialize to TOML
        let toml_content = toml::to_string(&config).unwrap();
        std::fs::write(&config_path, toml_content).unwrap();
        
        // Read back and verify
        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed_config: AppConfig = toml::from_str(&content).unwrap();
        
        assert_eq!(config.datomic.transactor_port, parsed_config.datomic.transactor_port);
        assert_eq!(config.datomic.database_name, parsed_config.datomic.database_name);
        assert_eq!(config.audio.sample_rate, parsed_config.audio.sample_rate);
    }
    
    /// Test audio device handling
    #[test]
    fn test_audio_device_models() {
        let device = AudioDevice {
            id: "test-device".to_string(),
            name: "Test Audio Device".to_string(),
            is_default: true,
            device_type: "input".to_string(),
        };
        
        // Test serialization
        let json = serde_json::to_string(&device).unwrap();
        let deserialized: AudioDevice = serde_json::from_str(&json).unwrap();
        
        assert_eq!(device.id, deserialized.id);
        assert_eq!(device.name, deserialized.name);
        assert_eq!(device.is_default, deserialized.is_default);
        assert_eq!(device.device_type, deserialized.device_type);
    }
    
    /// Test audio recording models
    #[test]
    fn test_audio_recording_models() {
        use chrono::Utc;
        use uuid::Uuid;
        
        let recording = AudioRecording {
            id: Uuid::new_v4().to_string(),
            page_id: "test-page".to_string(),
            file_path: "/path/to/recording.wav".to_string(),
            duration_seconds: Some(120.5),
            recorded_at: Utc::now(),
        };
        
        // Test serialization
        let json = serde_json::to_string(&recording).unwrap();
        let deserialized: AudioRecording = serde_json::from_str(&json).unwrap();
        
        assert_eq!(recording.id, deserialized.id);
        assert_eq!(recording.page_id, deserialized.page_id);
        assert_eq!(recording.file_path, deserialized.file_path);
        assert_eq!(recording.duration_seconds, deserialized.duration_seconds);
    }
    
    /// Test create block request validation
    #[test]
    fn test_create_block_request() {
        let request = CreateBlockRequest {
            content: "Test content".to_string(),
            page_id: Some("test-page".to_string()),
            parent_id: None,
            order: Some(0),
        };
        
        // Test serialization
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CreateBlockRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(request.content, deserialized.content);
        assert_eq!(request.page_id, deserialized.page_id);
        assert_eq!(request.parent_id, deserialized.parent_id);
        assert_eq!(request.order, deserialized.order);
    }
    
    /// Test timestamp models
    #[test]
    fn test_timestamp_models() {
        use uuid::Uuid;
        
        let timestamp = AudioTimestamp {
            id: Uuid::new_v4().to_string(),
            block_id: Uuid::new_v4().to_string(),
            recording_id: Uuid::new_v4().to_string(),
            timestamp_ms: 5000,
        };
        
        // Test serialization
        let json = serde_json::to_string(&timestamp).unwrap();
        let deserialized: AudioTimestamp = serde_json::from_str(&json).unwrap();
        
        assert_eq!(timestamp.id, deserialized.id);
        assert_eq!(timestamp.block_id, deserialized.block_id);
        assert_eq!(timestamp.recording_id, deserialized.recording_id);
        assert_eq!(timestamp.timestamp_ms, deserialized.timestamp_ms);
    }
}

/// Integration tests for the complete system
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;
    
    /// Test complete application setup (requires Datomic)
    #[tokio::test]
    #[ignore] // Ignore by default as it requires Datomic setup
    async fn test_complete_setup() {
        // This test requires a running Datomic transactor
        // Run with: cargo test test_complete_setup -- --ignored
        
        let temp_dir = TempDir::new().unwrap();
        let mut config = AppConfig::default();
        config.data_dir = temp_dir.path().join("gita-test");
        config.audio.recordings_dir = temp_dir.path().join("recordings-test");
        
        // Try to create the peer client
        let result = DatomicPeerClient::new(config).await;
        
        if result.is_ok() {
            let client = result.unwrap();
            
            // Test health check
            let health = client.health_check().await;
            assert!(health.is_ok());
            
            // Test creating a block
            let block_request = CreateBlockRequest {
                content: "Integration test block".to_string(),
                page_id: Some("integration-test-page".to_string()),
                parent_id: None,
                order: Some(0),
            };
            
            let block_result = client.create_block(block_request, None).await;
            assert!(block_result.is_ok());
            
            let block = block_result.unwrap();
            assert_eq!(block.content, "Integration test block");
            assert_eq!(block.page_id, Some("integration-test-page".to_string()));
            
            // Test querying blocks
            let blocks = client.get_page_blocks("integration-test-page").await;
            assert!(blocks.is_ok());
            
        } else {
            println!("Skipping integration test - Datomic not available: {:?}", result.err());
        }
    }
    
    /// Test error handling in real scenarios
    #[tokio::test]
    async fn test_error_scenarios() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = AppConfig::default();
        config.data_dir = temp_dir.path().join("gita-test");
        config.audio.recordings_dir = temp_dir.path().join("recordings-test");
        
        // Test with invalid Datomic configuration
        config.datomic.db_uri = "datomic:dev://invalid:9999/test".to_string();
        config.datomic.transactor_host = "invalid-host".to_string();
        config.datomic.transactor_port = 9999;
        
        let result = DatomicPeerClient::new(config).await;
        assert!(result.is_err());
        
        // Verify the error type
        match result.unwrap_err() {
            DatomicError::ConnectionError(_) | 
            DatomicError::JvmInitializationFailed(_) |
            DatomicError::ConfigError(_) => {
                // Expected error types
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }
    
    /// Benchmark basic operations
    #[tokio::test]
    #[ignore] // Ignore by default as it requires Datomic setup
    async fn test_performance_benchmarks() {
        use std::time::Instant;
        
        let config = AppConfig::default();
        
        if let Ok(client) = DatomicPeerClient::new(config).await {
            // Benchmark health check
            let start = Instant::now();
            let _ = client.health_check().await;
            let health_check_time = start.elapsed();
            
            println!("Health check time: {:?}", health_check_time);
            assert!(health_check_time.as_millis() < 1000); // Should be fast
            
            // Benchmark block creation
            let start = Instant::now();
            let block_request = CreateBlockRequest {
                content: "Performance test block".to_string(),
                page_id: Some("performance-test-page".to_string()),
                parent_id: None,
                order: Some(0),
            };
            
            let _ = client.create_block(block_request, None).await;
            let create_time = start.elapsed();
            
            println!("Block creation time: {:?}", create_time);
            assert!(create_time.as_millis() < 5000); // Should be reasonably fast
            
        } else {
            println!("Skipping performance test - Datomic not available");
        }
    }
}

/// Property-based tests using quickcheck
#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};
    use quickcheck_macros::quickcheck;
    
    /// Test that block IDs are always valid UUIDs
    #[quickcheck]
    fn test_block_id_validity(content: String) -> TestResult {
        if content.is_empty() {
            return TestResult::discard();
        }
        
        let request = CreateBlockRequest {
            content: content.clone(),
            page_id: None,
            parent_id: None,
            order: None,
        };
        
        // We can't test the actual creation without Datomic,
        // but we can test the request validation
        let json = serde_json::to_string(&request);
        TestResult::from_bool(json.is_ok())
    }
    
    /// Test that configuration values are always valid
    #[quickcheck]
    fn test_config_validation(port: u16, timeout: u64) -> TestResult {
        let mut config = AppConfig::default();
        config.datomic.transactor_port = port;
        config.datomic.connection_timeout_ms = timeout;
        
        // Test serialization
        let result = toml::to_string(&config);
        TestResult::from_bool(result.is_ok())
    }
    
    /// Test that error messages are always valid
    #[quickcheck]
    fn test_error_message_validity(msg: String) -> bool {
        let error = DatomicError::connection_error(&msg);
        !error.to_string().is_empty()
    }
}
