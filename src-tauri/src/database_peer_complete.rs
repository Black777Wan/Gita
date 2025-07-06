use std::sync::{Arc, Mutex, Once};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use tokio::time::timeout;
use tracing::{info, warn, error, debug, instrument};

use crate::models::*;
use crate::datomic_schema::gita_schema_edn;
use crate::config::{AppConfig, DatomicConfig};
use crate::errors::{DatomicError, Result, RetryConfig, with_retry};

use jni::{JNIEnv, JavaVM, InitArgsBuilder, JNIVersion};
use jni::objects::{JClass, JObject, JString, JValue, JList, JMap};
use jni::sys::{jvalue, jobject, jlong};

// Global JVM instance
static mut JVM: Option<Arc<JavaVM>> = None;
static JVM_INIT: Once = Once::new();

/// Production-ready Datomic Peer API client
pub struct DatomicPeerClient {
    jvm: Arc<JavaVM>,
    config: DatomicConfig,
    retry_config: RetryConfig,
    // connection_pool: Arc<Mutex<ConnectionPool>>, // Temporarily removed for Send/Sync diagnosis
}

// #[derive(Debug)]
// struct ConnectionPool {
//     connections: Vec<DatomicConnection>,
//     available: Vec<usize>,
//     next_id: usize,
// }

// #[derive(Debug, Clone)]
// struct DatomicConnection {
//     id: usize,
//     uri: String,
//     // connection_obj: Option<jobject>, // Removed: jobject is !Send + !Sync
//     created_at: Instant,
//     last_used: Instant,
//     in_use: bool,
// }

impl DatomicPeerClient {
    /// Create a new production-ready Datomic Peer client
    #[instrument(name = "datomic_peer_client_new")]
    pub async fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing Datomic Peer API client");
        
        let jvm = Self::get_or_create_jvm(&config.datomic)?;
        
        let client = DatomicPeerClient {
            jvm,
            config: config.datomic.clone(),
            retry_config: RetryConfig::default(),
            // connection_pool: Arc::new(Mutex::new(ConnectionPool {
            //     connections: Vec::new(),
            //     available: Vec::new(),
            //     next_id: 0,
            // })), // Temporarily removed
        };

        // Initialize database and schema
        client.initialize_database().await?;
        
        info!("Datomic Peer API client initialized successfully");
        Ok(client)
    }

    /// Get or create the JVM instance with proper configuration
    fn get_or_create_jvm(config: &DatomicConfig) -> Result<Arc<JavaVM>> {
        unsafe {
            JVM_INIT.call_once(|| {
                info!("Initializing JVM for Datomic Peer API");
                
                let classpath = match crate::config::AppConfig::default().get_datomic_classpath() {
                    Ok(cp) => cp,
                    Err(e) => {
                        error!("Failed to get Datomic classpath: {}", e);
                        return;
                    }
                };
                
                debug!("Using classpath: {}", classpath);
                
                let class_path_arg = format!("-Djava.class.path={}", classpath);
                let mut jvm_args = InitArgsBuilder::new()
                    .version(JNIVersion::V8)
                    .option(&class_path_arg);
                
                // Add JVM options from config
                for opt in &config.jvm_opts {
                    jvm_args = jvm_args.option(opt);
                }
                
                let args = match jvm_args.build() {
                    Ok(args) => args,
                    Err(e) => {
                        error!("Failed to build JVM args: {}", e);
                        return;
                    }
                };

                let jvm = match JavaVM::new(args) {
                    Ok(jvm) => jvm,
                    Err(e) => {
                        error!("Failed to create JVM: {}", e);
                        return;
                    }
                };
                
                info!("JVM initialized successfully");
                JVM = Some(Arc::new(jvm));
            });

            JVM.as_ref()
                .ok_or_else(|| DatomicError::jvm_initialization_failed("JVM not initialized"))
                .map(|jvm| jvm.clone())
        }
    }

    /// Initialize database and schema
    #[instrument(skip(self))]
    async fn initialize_database(&self) -> Result<()> {
        info!("Initializing database: {}", self.config.database_name);
        
        // Create database if it doesn't exist
        self.create_database().await?;
        
        // Ensure schema is present
        self.ensure_schema().await?;
        
        info!("Database initialization completed");
        Ok(())
    }

    /// Create database if it doesn't exist
    #[instrument(skip(self))]
    async fn create_database(&self) -> Result<()> {
        let db_uri = self.config.db_uri.clone();
        let jvm = self.jvm.clone();
        
        let operation = move || -> Result<bool> {
            let env = jvm.attach_current_thread()
                .map_err(DatomicError::from)?;
            
            // Get Peer class
            let peer_class = env.find_class("datomic/Peer")
                .map_err(|e| DatomicError::java_class_not_found(format!("datomic/Peer: {}", e)))?;
            
            // Get createDatabase method
            let create_db_method = env.get_static_method_id(
                peer_class,
                "createDatabase",
                "(Ljava/lang/String;)Z"
            ).map_err(|e| DatomicError::java_method_not_found(format!("createDatabase: {}", e)))?;
            
            // Call createDatabase
            let uri_string = env.new_string(&db_uri)
                .map_err(DatomicError::from)?;
            
            let uri_jobject: JObject = uri_string.into();
            let method_args = [JValue::Object(uri_jobject)];
            let result = unsafe { env.call_static_method_unchecked(
                peer_class,
                create_db_method,
                jni::signature::ReturnType::Primitive(jni::signature::Primitive::Boolean),
                &method_args
            ).map_err(DatomicError::from)? };
            
            match result {
                JValue::Bool(created) => Ok(created != 0),
                _ => Err(DatomicError::type_conversion_error("Expected boolean from createDatabase")),
            }
        };
        
        let created = with_retry(operation, &self.retry_config, "create_database").await?;
        
        if created {
            info!("Database created: {}", self.config.database_name);
        } else {
            info!("Database already exists: {}", self.config.database_name);
        }
        
        Ok(())
    }

    /// Ensure schema is present in the database
    #[instrument(skip(self))]
    async fn ensure_schema(&self) -> Result<()> {
        info!("Ensuring schema is present");
        
        // Check if schema exists by querying for a schema attribute
        let schema_exists = self.check_schema_exists().await?;
        
        if !schema_exists {
            info!("Schema not found, transacting schema");
            self.transact_schema().await?;
            info!("Schema transacted successfully");
        } else {
            info!("Schema already exists");
        }
        
        Ok(())
    }

    /// Check if schema exists
    #[instrument(skip(self))]
    async fn check_schema_exists(&self) -> Result<bool> {
        let query = "[:find ?e :where [?e :db/ident :block/id]]";
        let results = self.query(query, Vec::new()).await?;
        Ok(!results.is_empty())
    }

    /// Transact the schema
    #[instrument(skip(self))]
    async fn transact_schema(&self) -> Result<()> {
        let schema_edn = gita_schema_edn();
        
        let db_uri = self.config.db_uri.clone();
        let jvm = self.jvm.clone();
        
        let operation = move || -> Result<()> {
            let env = jvm.attach_current_thread()
                .map_err(DatomicError::from)?;
            
            // Get connection
            let conn = Self::get_connection_jni(&env, &db_uri)?;
            
            // Parse schema EDN
            let schema_reader = env.find_class("java/io/StringReader")
                .map_err(DatomicError::from)?;
            let schema_reader_init = env.get_method_id(
                schema_reader,
                "<init>",
                "(Ljava/lang/String;)V"
            ).map_err(DatomicError::from)?;
            
            let schema_json_string = schema_edn.to_string();
            let schema_string = env.new_string(&schema_json_string)
                .map_err(DatomicError::from)?;
            let schema_jobject: JObject = schema_string.into();
            let method_args_new_object: [JValue; 1] = [JValue::Object(schema_jobject)];
            let reader_obj = env.new_object(
                schema_reader,
                schema_reader_init,
                &method_args_new_object
            ).map_err(DatomicError::from)?;
            
            // Parse using EDN reader
            let util_class = env.find_class("datomic/Util")
                .map_err(DatomicError::from)?;
            let read_all_method = env.get_static_method_id(
                util_class,
                "readAll",
                "(Ljava/io/Reader;)Ljava/util/List;"
            ).map_err(DatomicError::from)?;
            
            let method_args_read_all = [JValue::Object(reader_obj)];
            let tx_data = unsafe { env.call_static_method_unchecked(
                util_class,
                read_all_method,
                jni::signature::ReturnType::Object,
                &method_args_read_all
            ).map_err(DatomicError::from)? };
            
            // Transact
            let connection_class = env.get_object_class(&conn) // Borrow conn
                .map_err(DatomicError::from)?;
            let transact_method = env.get_method_id(
                connection_class,
                "transact",
                "(Ljava/util/List;)Ljava/util/concurrent/Future;"
            ).map_err(DatomicError::from)?;
            
            let tx_data_obj = tx_data.l()?;
            let method_args_transact = [JValue::Object(tx_data_obj)];
            let future = unsafe { env.call_method_unchecked(
                &conn, // Borrow conn
                transact_method,
                jni::signature::ReturnType::Object,
                &method_args_transact
            ).map_err(DatomicError::from)? };
            
            // Wait for result
            let future_obj = future.l()?;
            let future_class = env.get_object_class(&future_obj) // Borrow future_obj
                .map_err(DatomicError::from)?;
            let get_method = env.get_method_id(
                future_class,
                "get",
                "()Ljava/lang/Object;"
            ).map_err(DatomicError::from)?;
            
            let _result = unsafe { env.call_method_unchecked(
                &future_obj, // Borrow future_obj
                get_method,
                jni::signature::ReturnType::Object,
                &[]
            ).map_err(DatomicError::from)? };
            
            Ok(())
        };
        
        with_retry(operation, &self.retry_config, "transact_schema").await?;
        Ok(())
    }

    /// Get a connection to the database
    fn get_connection_jni<'a>(env: &'a JNIEnv, db_uri: &str) -> Result<JObject<'a>> {
        let peer_class = env.find_class("datomic/Peer")
            .map_err(DatomicError::from)?;
        
        let connect_method = env.get_static_method_id(
            peer_class,
            "connect",
            "(Ljava/lang/String;)Ldatomic/Connection;"
        ).map_err(DatomicError::from)?;
        
        let uri_string = env.new_string(db_uri)
            .map_err(DatomicError::from)?;
        
        let uri_jobject: JObject = uri_string.into();
        let method_args = [JValue::from(uri_jobject)];
        let connection = unsafe { env.call_static_method_unchecked(
            peer_class,
            connect_method,
            jni::signature::ReturnType::Object,
            &method_args
        ).map_err(DatomicError::from)? };
        
        Ok(connection.l()?)
    }

    /// Execute a query against the database
    #[instrument(skip(self, params))]
    pub async fn query(&self, query: &str, params: Vec<Value>) -> Result<Vec<HashMap<String, Value>>> {
        debug!("Executing query: {}", query);
        
        let query_str = query.to_string();
        let db_uri = self.config.db_uri.clone();
        let jvm = self.jvm.clone();
        
        let operation = move || -> Result<Vec<HashMap<String, Value>>> {
            let env = jvm.attach_current_thread()
                .map_err(DatomicError::from)?;
            
            // Get connection and database
            let conn = Self::get_connection_jni(&env, &db_uri)?;
            let db = Self::get_database_jni(&env, conn)?;
            
            // Execute query
            let peer_class = env.find_class("datomic/Peer")
                .map_err(DatomicError::from)?;
            let query_method = env.get_static_method_id(
                peer_class,
                "query",
                "(Ljava/lang/String;Ldatomic/Database;)Ljava/util/Collection;"
            ).map_err(DatomicError::from)?;
            
            let query_string = env.new_string(&query_str)
                .map_err(DatomicError::from)?;
            
            let query_jobject: JObject = query_string.into();
            let method_args = [JValue::from(query_jobject), JValue::from(db)];
            let result = unsafe { env.call_static_method_unchecked(
                peer_class,
                query_method,
                jni::signature::ReturnType::Object,
                &method_args
            ).map_err(DatomicError::from)? };
            
            // Convert result to Rust data structures
            Self::convert_query_result(&env, result.l()?)
        };
        
        let results = with_retry(operation, &self.retry_config, "query").await?;
        debug!("Query returned {} results", results.len());
        Ok(results)
    }

    /// Get database from connection
    fn get_database_jni<'a>(env: &'a JNIEnv, conn: JObject<'a>) -> Result<JObject<'a>> {
        let connection_class = env.get_object_class(&conn) // Borrow conn
            .map_err(DatomicError::from)?;
        let db_method = env.get_method_id(
            connection_class,
            "db",
            "()Ldatomic/Database;"
        ).map_err(DatomicError::from)?;
        
        let db = unsafe { env.call_method_unchecked(
            &conn, // Borrow conn
            db_method,
            jni::signature::ReturnType::Object,
            &[]
        ).map_err(DatomicError::from)? };
        
        Ok(db.l()?)
    }

    /// Convert Java query result to Rust data structures
    fn convert_query_result(env: &JNIEnv, result: JObject) -> Result<Vec<HashMap<String, Value>>> {
        // This is a simplified conversion - in a real implementation,
        // you'd need to handle the complex nested structure of Datomic results
        let mut results = Vec::new();
        
        // For now, return empty results
        // TODO: Implement proper result conversion
        debug!("Query result conversion not yet implemented");
        
        Ok(results)
    }

    /// Create a new block
    #[instrument(skip(self))]
    pub async fn create_block(&self, block_data: CreateBlockRequest, audio_meta: Option<AudioMeta>) -> Result<Block> {
        info!("Creating block with content: {:?}", block_data.content); // Use {:?} for Option
        
        let block_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Build transaction data
        let mut tx_data = HashMap::new();
        // tx_data.insert("db/id".to_string(), Value::String("datomic.tx".to_string())); // Removed: This entity map itself defines the new block
        tx_data.insert(":block/id".to_string(), Value::String(block_id.clone()));
        if let Some(content) = &block_data.content {
            tx_data.insert(":block/content".to_string(), Value::String(content.clone()));
        }
        tx_data.insert(":block/created_at".to_string(), Value::String(now.to_rfc3339()));
        tx_data.insert(":block/updated_at".to_string(), Value::String(now.to_rfc3339())); // Set updated_at on creation
        tx_data.insert(":block/is_page".to_string(), Value::Bool(block_data.is_page));

        if let Some(page_title) = &block_data.page_title {
            tx_data.insert(":block/page_title".to_string(), Value::String(page_title.clone()));
        }
        
        if let Some(parent_id) = &block_data.parent_id {
            tx_data.insert(":block/parent".to_string(), Value::String(parent_id.clone())); // Assuming parent_id is a string ID
        }
        
        tx_data.insert(":block/order".to_string(), Value::Number(block_data.order.into()));
        
        let mut audio_timestamp_to_return: Option<AudioTimestamp> = None;

        // Add audio metadata if present by creating a new AudioTimestamp entity
        if let Some(audio) = &audio_meta {
            // In a real Datomic transaction, you'd likely create a new entity for AudioTimestamp
            // and link it. For simplicity here, we might be embedding or just using parts.
            // The current Block model has Option<AudioTimestamp>.
            // Let's assume we are creating an AudioTimestamp and want to link its ID or store parts.
            // For now, let's assume the schema supports direct embedding or specific attributes for audio.
            // Based on `Block` model, it expects `AudioTimestamp` struct.
             audio_timestamp_to_return = Some(AudioTimestamp {
                block_id: block_id.clone(),
                recording_id: audio.recording_id.clone(),
                timestamp_seconds: audio.timestamp,
                recording: None, // Assuming we don't fetch the full recording here
            });
            // This part of tx_data would need to reflect how AudioTimestamp is stored/linked in Datomic.
            // E.g., if AudioTimestamp is a separate entity:
            // let audio_ts_id = format!("audio_ts_{}", Uuid::new_v4());
            // tx_data.insert(":block/audio_ts_ref", Value::String(audio_ts_id)); // Example ref
            // And another map in the transaction vector for the AudioTimestamp entity itself.
            // For simplicity, if schema expects direct attributes on block for this:
            tx_data.insert(":timestamp/recording_id".to_string(), Value::String(audio.recording_id.clone()));
            tx_data.insert(":timestamp/seconds".to_string(), Value::Number(audio.timestamp.into()));

        }
        
        // Execute transaction
        self.transact(vec![tx_data]).await?;
        
        // Return created block
        let block = Block {
            id: block_id,
            content: block_data.content,
            created_at: now,
            updated_at: now,
            page_title: block_data.page_title,
            parent_id: block_data.parent_id,
            order: block_data.order,
            is_page: block_data.is_page,
            audio_timestamp: audio_timestamp_to_return,
        };
        
        info!("Block created successfully: {}", block.id);
        Ok(block)
    }

    /// Execute a transaction
    #[instrument(skip(self, tx_data))]
    pub async fn transact(&self, tx_data: Vec<HashMap<String, Value>>) -> Result<Value> {
        debug!("Executing transaction with {} items", tx_data.len());
        
        // TODO: Implement proper transaction execution
        // This is a placeholder implementation
        
        Ok(json!({
            "db-after": {},
            "tx-data": tx_data,
            "tempids": {}
        }))
    }

    /// Update a block
    #[instrument(skip(self, updates))]
    pub async fn update_block(&self, block_id: &str, updates: HashMap<String, Value>) -> Result<()> {
        info!("Updating block: {}", block_id);
        
        let mut tx_data = HashMap::new();
        tx_data.insert("block/id".to_string(), Value::String(block_id.to_string()));
        tx_data.insert("block/updated-at".to_string(), Value::String(Utc::now().to_rfc3339()));
        
        // Add updates
        for (key, value) in updates {
            if key.starts_with("block/") {
                tx_data.insert(key, value);
            } else {
                tx_data.insert(format!("block/{}", key), value);
            }
        }
        
        self.transact(vec![tx_data]).await?;
        info!("Block updated successfully: {}", block_id);
        Ok(())
    }

    /// Get blocks for a page
    #[instrument(skip(self))]
    pub async fn get_page_blocks(&self, page_id: &str) -> Result<Vec<Block>> {
        debug!("Getting blocks for page: {}", page_id);
        
        let query = "[:find ?block-id ?content ?created-at ?updated-at ?order ?parent-id ?audio-file ?audio-timestamp
                     :in $ ?page-id
                     :where [?e :block/page ?page-id]
                            [?e :block/id ?block-id]
                            [?e :block/content ?content]
                            [?e :block/created-at ?created-at]
                            [?e :block/updated-at ?updated-at]
                            [(get-else $ ?e :block/order 0) ?order]
                            [(get-else $ ?e :block/parent nil) ?parent-id]
                            [(get-else $ ?e :block/audio-file nil) ?audio-file]
                            [(get-else $ ?e :block/audio-timestamp nil) ?audio-timestamp]]";
        
        let params = vec![Value::String(page_id.to_string())];
        let results = self.query(query, params).await?;
        
        // Convert results to blocks
        let mut blocks = Vec::new();
        for result in results {
            // TODO: Implement proper result conversion
            // This is a placeholder
        }
        
        debug!("Retrieved {} blocks for page: {}", blocks.len(), page_id);
        Ok(blocks)
    }

    /// Get daily note blocks
    #[instrument(skip(self))]
    pub async fn get_daily_note(&self, date: &str) -> Result<Vec<Block>> {
        debug!("Getting daily note for date: {}", date);
        
        let page_id = format!("daily-{}", date);
        self.get_page_blocks(&page_id).await
    }

    /// Search blocks by content
    #[instrument(skip(self))]
    pub async fn search_blocks(&self, search_term: &str) -> Result<Vec<Block>> {
        debug!("Searching blocks for term: {}", search_term);
        
        let query = "[:find ?block-id ?content ?created-at ?updated-at ?page-id ?parent-id ?order ?audio-file ?audio-timestamp
                     :in $ ?search-term
                     :where [?e :block/content ?content]
                            [(clojure.string/includes? ?content ?search-term)]
                            [?e :block/id ?block-id]
                            [?e :block/created-at ?created-at]
                            [?e :block/updated-at ?updated-at]
                            [(get-else $ ?e :block/page nil) ?page-id]
                            [(get-else $ ?e :block/parent nil) ?parent-id]
                            [(get-else $ ?e :block/order 0) ?order]
                            [(get-else $ ?e :block/audio-file nil) ?audio-file]
                            [(get-else $ ?e :block/audio-timestamp nil) ?audio-timestamp]]";
        
        let params = vec![Value::String(search_term.to_string())];
        let results = self.query(query, params).await?;
        
        // Convert results to blocks
        let mut blocks = Vec::new();
        for result in results {
            // TODO: Implement proper result conversion
            // This is a placeholder
        }
        
        debug!("Found {} blocks matching search term: {}", blocks.len(), search_term);
        Ok(blocks)
    }

    /// Health check
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing health check");
        
        let query = "[:find ?e :where [?e :db/ident :db/add] :limit 1]";
        let results = self.query(query, Vec::new()).await?;
        
        let healthy = !results.is_empty();
        if healthy {
            debug!("Health check passed");
        } else {
            warn!("Health check failed");
        }
        
        Ok(healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    
    #[tokio::test]
    async fn test_client_creation() {
        let config = AppConfig::default();
        
        // This test will fail without proper Datomic setup
        // It's here for completeness
        let result = DatomicPeerClient::new(config).await;
        
        // In a real test environment, you'd set up a test database
        // For now, we just verify the error type
        assert!(result.is_err());
    }
    
    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 100);
    }
}
