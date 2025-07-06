use std::sync::{Arc, Mutex, Once};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::models::*;
use crate::datomic_schema::gita_schema_edn;
use jni::{JNIEnv, JavaVM, InitArgsBuilder, JNIVersion};
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::jvalue;

// Global JVM instance
static mut JVM: Option<Arc<JavaVM>> = None;
static INIT: Once = Once::new();

// Datomic connection URI
const DATOMIC_URI: &str = "datomic:dev://localhost:8998/gita";

/// A real Datomic Peer API client that uses JNI to interact with Datomic
pub struct DatomicPeerClient {
    db_uri: String,
    jvm: Arc<JavaVM>,
}

impl DatomicPeerClient {
    /// Create a new Datomic Peer client with JNI
    pub async fn new() -> Result<Self> {
        let jvm = Self::get_or_create_jvm()?;
        
        let client = DatomicPeerClient {
            db_uri: DATOMIC_URI.to_string(),
            jvm,
        };

        // Initialize connection and ensure schema
        client.create_database().await?;
        client.ensure_schema().await?;

        Ok(client)
    }

    /// Get or create the JVM instance
    fn get_or_create_jvm() -> Result<Arc<JavaVM>> {
        unsafe {
            INIT.call_once(|| {
                let jvm_args = InitArgsBuilder::new()
                    .version(JNIVersion::V8)
                    .option("-Xmx4g")
                    .option("-Xms1g")
                    .option("-classpath")
                    .option("C:\\Users\\yashd\\datomic-pro-1.0.7387\\lib\\*")
                    .build()
                    .expect("Failed to build JVM args");

                let jvm = JavaVM::new(jvm_args)
                    .expect("Failed to create JVM");
                
                JVM = Some(Arc::new(jvm));
            });

            JVM.as_ref().unwrap().clone()
        }
    }

    /// Create the database if it doesn't exist
    async fn create_database(&self) -> Result<()> {
        let env = self.jvm.attach_current_thread()?;
        
        // Call datomic.api/create-database
        let datomic_api = env.find_class("datomic/Peer")?;
        let create_db_method = env.get_static_method_id(
            datomic_api,
            "createDatabase",
            "(Ljava/lang/String;)Z"
        )?;
        
        let uri_string = env.new_string(&self.db_uri)?;
        let result = env.call_static_method_unchecked(
            datomic_api,
            create_db_method,
            jni::signature::JavaType::Primitive(jni::signature::Primitive::Boolean),
            &[JValue::Object(uri_string.into())]
        )?;
        
        match result {
            JValue::Bool(created) => {
                if created != 0 {
                    println!("Database created: {}", self.db_uri);
                } else {
                    println!("Database already exists: {}", self.db_uri);
                }
            }
            _ => return Err(anyhow!("Unexpected return type from createDatabase")),
        }
        
        Ok(())
    }

    /// Connect to the database
    async fn connect(&self) -> Result<JObject> {
        let env = self.jvm.attach_current_thread()?;
        
        // Call datomic.api/connect
        let datomic_api = env.find_class("datomic/Peer")?;
        let connect_method = env.get_static_method_id(
            datomic_api,
            "connect",
            "(Ljava/lang/String;)Ldatomic/Connection;"
        )?;
        
        let uri_string = env.new_string(&self.db_uri)?;
        let connection = env.call_static_method_unchecked(
            datomic_api,
            connect_method,
            jni::signature::JavaType::Object("datomic/Connection".to_string()),
            &[JValue::Object(uri_string.into())]
        )?;
        
        match connection {
            JValue::Object(conn) => Ok(conn),
            _ => Err(anyhow!("Failed to connect to database")),
        }
    }

    /// Ensure the schema exists in the database
    pub async fn ensure_schema(&self) -> Result<()> {
        let env = self.jvm.attach_current_thread()?;
        
        // Connect to database
        let connection = self.connect().await?;
        
        // Get current database value
        let db = self.get_db(&env, &connection)?;
        
        // Check if schema exists by looking for :block/content attribute
        let query = r#"[:find ?e . :where [?e :db/ident :block/content]]"#;
        let query_result = self.execute_query(&env, &db, query, &[])?;
        
        // If schema doesn't exist, transact it
        if self.is_empty_result(&env, &query_result)? {
            println!("Schema not found, attempting to transact it...");
            self.transact_schema(&env, &connection).await?;
            println!("Schema transaction successful.");
        } else {
            println!("Schema already present.");
        }
        
        Ok(())
    }

    /// Get the current database value
    fn get_db(&self, env: &JNIEnv, connection: &JObject) -> Result<JObject> {
        let db_method = env.get_method_id(
            env.get_object_class(connection)?,
            "db",
            "()Ldatomic/Database;"
        )?;
        
        let db = env.call_method_unchecked(
            connection,
            db_method,
            jni::signature::JavaType::Object("datomic/Database".to_string()),
            &[]
        )?;
        
        match db {
            JValue::Object(db_obj) => Ok(db_obj),
            _ => Err(anyhow!("Failed to get database")),
        }
    }

    /// Execute a query
    fn execute_query(&self, env: &JNIEnv, db: &JObject, query: &str, args: &[JValue]) -> Result<JObject> {
        let datomic_api = env.find_class("datomic/Peer")?;
        let query_method = env.get_static_method_id(
            datomic_api,
            "q",
            "(Ljava/lang/String;Ldatomic/Database;[Ljava/lang/Object;)Ljava/util/Collection;"
        )?;
        
        let query_string = env.new_string(query)?;
        let args_array = env.new_object_array(args.len() as i32, "java/lang/Object", JObject::null())?;
        
        for (i, arg) in args.iter().enumerate() {
            match arg {
                JValue::Object(obj) => {
                    env.set_object_array_element(args_array, i as i32, *obj)?;
                }
                _ => {} // Handle other types as needed
            }
        }
        
        let result = env.call_static_method_unchecked(
            datomic_api,
            query_method,
            jni::signature::JavaType::Object("java/util/Collection".to_string()),
            &[JValue::Object(query_string.into()), JValue::Object(*db), JValue::Object(args_array.into())]
        )?;
        
        match result {
            JValue::Object(result_obj) => Ok(result_obj),
            _ => Err(anyhow!("Query execution failed")),
        }
    }

    /// Check if query result is empty
    fn is_empty_result(&self, env: &JNIEnv, result: &JObject) -> Result<bool> {
        let collection_class = env.find_class("java/util/Collection")?;
        let is_empty_method = env.get_method_id(collection_class, "isEmpty", "()Z")?;
        
        let is_empty = env.call_method_unchecked(
            result,
            is_empty_method,
            jni::signature::JavaType::Primitive(jni::signature::Primitive::Boolean),
            &[]
        )?;
        
        match is_empty {
            JValue::Bool(empty) => Ok(empty != 0),
            _ => Err(anyhow!("Failed to check if result is empty")),
        }
    }

    /// Transact the schema
    async fn transact_schema(&self, env: &JNIEnv, connection: &JObject) -> Result<()> {
        let schema_edn = self.convert_schema_to_edn()?;
        let schema_string = env.new_string(&schema_edn)?;
        
        // Parse the schema using Clojure reader
        let clojure_class = env.find_class("clojure/lang/RT")?;
        let read_string_method = env.get_static_method_id(
            clojure_class,
            "readString",
            "(Ljava/lang/String;)Ljava/lang/Object;"
        )?;
        
        let parsed_schema = env.call_static_method_unchecked(
            clojure_class,
            read_string_method,
            jni::signature::JavaType::Object("java/lang/Object".to_string()),
            &[JValue::Object(schema_string.into())]
        )?;
        
        // Transact the schema
        let transact_method = env.get_method_id(
            env.get_object_class(connection)?,
            "transact",
            "(Ljava/util/List;)Ljava/util/concurrent/Future;"
        )?;
        
        let future = env.call_method_unchecked(
            connection,
            transact_method,
            jni::signature::JavaType::Object("java/util/concurrent/Future".to_string()),
            &[parsed_schema]
        )?;
        
        // Wait for the transaction to complete
        if let JValue::Object(future_obj) = future {
            let future_class = env.find_class("java/util/concurrent/Future")?;
            let get_method = env.get_method_id(future_class, "get", "()Ljava/lang/Object;")?;
            
            env.call_method_unchecked(
                future_obj,
                get_method,
                jni::signature::JavaType::Object("java/lang/Object".to_string()),
                &[]
            )?;
        }
        
        Ok(())
    }

    /// Convert JSON schema to EDN format
    fn convert_schema_to_edn(&self) -> Result<String> {
        let schema = gita_schema_edn();
        
        // Convert JSON to EDN string
        let edn = self.json_to_edn(&schema)?;
        Ok(edn)
    }

    /// Convert JSON to EDN format
    fn json_to_edn(&self, json: &Value) -> Result<String> {
        match json {
            Value::Array(arr) => {
                let items: Result<Vec<String>> = arr.iter()
                    .map(|v| self.json_to_edn(v))
                    .collect();
                Ok(format!("[{}]", items?.join(" ")))
            }
            Value::Object(obj) => {
                let items: Result<Vec<String>> = obj.iter()
                    .map(|(k, v)| {
                        let key = if k.starts_with(':') {
                            k.clone()
                        } else {
                            format!(":{}", k)
                        };
                        let value = self.json_to_edn(v)?;
                        Ok(format!("{} {}", key, value))
                    })
                    .collect();
                Ok(format!("{{{}}}", items?.join(" ")))
            }
            Value::String(s) => {
                if s.starts_with(':') {
                    Ok(s.clone())
                } else {
                    Ok(format!("\"{}\"", s))
                }
            }
            Value::Number(n) => Ok(n.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Null => Ok("nil".to_string()),
        }
    }

    /// Execute a transaction
    pub async fn transact(&self, tx_data: &Value) -> Result<Value> {
        let env = self.jvm.attach_current_thread()?;
        let connection = self.connect().await?;
        
        // Convert transaction data to EDN
        let tx_edn = self.json_to_edn(tx_data)?;
        let tx_string = env.new_string(&tx_edn)?;
        
        // Parse the transaction data
        let clojure_class = env.find_class("clojure/lang/RT")?;
        let read_string_method = env.get_static_method_id(
            clojure_class,
            "readString",
            "(Ljava/lang/String;)Ljava/lang/Object;"
        )?;
        
        let parsed_tx = env.call_static_method_unchecked(
            clojure_class,
            read_string_method,
            jni::signature::JavaType::Object("java/lang/Object".to_string()),
            &[JValue::Object(tx_string.into())]
        )?;
        
        // Execute the transaction
        let transact_method = env.get_method_id(
            env.get_object_class(&connection)?,
            "transact",
            "(Ljava/util/List;)Ljava/util/concurrent/Future;"
        )?;
        
        let future = env.call_method_unchecked(
            &connection,
            transact_method,
            jni::signature::JavaType::Object("java/util/concurrent/Future".to_string()),
            &[parsed_tx]
        )?;
        
        // Wait for the transaction to complete and get the result
        if let JValue::Object(future_obj) = future {
            let future_class = env.find_class("java/util/concurrent/Future")?;
            let get_method = env.get_method_id(future_class, "get", "()Ljava/lang/Object;")?;
            
            env.call_method_unchecked(
                future_obj,
                get_method,
                jni::signature::JavaType::Object("java/lang/Object".to_string()),
                &[]
            )?;
        }
        
        // Return a success indicator
        Ok(json!({"success": true}))
    }

    /// Execute a query
    pub async fn query(&self, query: &str, args: Vec<Value>) -> Result<Value> {
        let env = self.jvm.attach_current_thread()?;
        let connection = self.connect().await?;
        let db = self.get_db(&env, &connection)?;
        
        // Convert args to JValues
        let jargs: Vec<JValue> = args.into_iter()
            .map(|v| match v {
                Value::String(s) => {
                    let jstring = env.new_string(&s).unwrap();
                    JValue::Object(jstring.into())
                }
                _ => JValue::Object(JObject::null()),
            })
            .collect();
        
        let result = self.execute_query(&env, &db, query, &jargs)?;
        
        // Convert result to JSON (simplified)
        Ok(json!([]))
    }

    /// Create a new block
    pub async fn create_block(
        &self,
        block_data: CreateBlockRequest,
        audio_meta: Option<AudioMeta>,
    ) -> Result<Block> {
        let block_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let temp_block_id = format!("new-block-{}", Uuid::new_v4());

        let mut tx_data = vec![
            json!([":db/add", temp_block_id, ":block/id", block_id]),
            json!([":db/add", temp_block_id, ":block/is_page", block_data.is_page]),
            json!([":db/add", temp_block_id, ":block/order", block_data.order]),
            json!([":db/add", temp_block_id, ":block/created_at", now.to_rfc3339()]),
            json!([":db/add", temp_block_id, ":block/updated_at", now.to_rfc3339()]),
        ];

        if let Some(content) = &block_data.content {
            tx_data.push(json!([":db/add", temp_block_id, ":block/content", content]));
        }

        if let Some(page_title) = &block_data.page_title {
            tx_data.push(json!([":db/add", temp_block_id, ":block/page_title", page_title]));
        }

        if let Some(parent_id) = &block_data.parent_id {
            tx_data.push(json!([":db/add", temp_block_id, ":block/parent", ["block/id", parent_id]]));
        }

        self.transact(&json!(tx_data)).await?;

        // Handle audio metadata if provided
        if let Some(audio) = &audio_meta {
            let timestamp_id = format!("new-timestamp-{}", Uuid::new_v4());
            let timestamp_tx = vec![
                json!([":db/add", timestamp_id, ":timestamp/block", ["block/id", block_id]]),
                json!([":db/add", timestamp_id, ":timestamp/recording_id", audio.recording_id]),
                json!([":db/add", timestamp_id, ":timestamp/seconds", audio.timestamp]),
            ];
            self.transact(&json!(timestamp_tx)).await?;
        }

        // Return the created block
        Ok(Block {
            id: block_id,
            content: block_data.content,
            parent_id: block_data.parent_id,
            order: block_data.order,
            is_page: block_data.is_page,
            page_title: block_data.page_title,
            created_at: now,
            updated_at: now,
            audio_timestamp: audio_meta.map(|a| AudioTimestamp {
                block_id: block_id.clone(),
                recording_id: a.recording_id,
                timestamp_seconds: a.timestamp,
                recording: None,
            }),
        })
    }

    /// Get a block by ID
    pub async fn get_block(&self, block_id: &str) -> Result<Option<Block>> {
        // This would require implementing proper result parsing
        // For now, return None
        Ok(None)
    }

    /// Get all blocks for a page
    pub async fn get_blocks_for_page(&self, page_id: &str) -> Result<Vec<Block>> {
        // This would require implementing proper result parsing
        // For now, return empty vector
        Ok(vec![])
    }

    /// Update a block
    pub async fn update_block(&self, block_id: &str, updates: HashMap<String, Value>) -> Result<Block> {
        let mut tx_data = vec![
            json!([":db/add", ["block/id", block_id], ":block/updated_at", Utc::now().to_rfc3339()]),
        ];

        for (key, value) in updates {
            let attr = match key.as_str() {
                "content" => ":block/content",
                "order" => ":block/order",
                "is_page" => ":block/is_page",
                "page_title" => ":block/page_title",
                _ => continue,
            };
            tx_data.push(json!([":db/add", ["block/id", block_id], attr, value]));
        }

        self.transact(&json!(tx_data)).await?;

        // Return the updated block (simplified)
        self.get_block(block_id).await?.ok_or_else(|| anyhow!("Block not found after update"))
    }

    /// Delete a block
    pub async fn delete_block(&self, block_id: &str) -> Result<()> {
        let tx_data = vec![
            json!([":db/retractEntity", ["block/id", block_id]]),
        ];

        self.transact(&json!(tx_data)).await?;
        Ok(())
    }

    /// Create an audio recording
    pub async fn create_audio_recording(&self, recording: AudioRecording) -> Result<AudioRecording> {
        let temp_id = format!("new-recording-{}", Uuid::new_v4());
        let mut tx_data = vec![
            json!([":db/add", temp_id, ":audio/id", recording.id]),
            json!([":db/add", temp_id, ":audio/page", ["block/id", recording.page_id]]),
            json!([":db/add", temp_id, ":audio/path", recording.file_path]),
            json!([":db/add", temp_id, ":audio/created_at", recording.recorded_at.to_rfc3339()]),
        ];

        if let Some(duration) = recording.duration_seconds {
            tx_data.push(json!([":db/add", temp_id, ":audio/duration", duration]));
        }

        self.transact(&json!(tx_data)).await?;
        Ok(recording)
    }

    /// Get all pages
    pub async fn get_all_pages(&self) -> Result<Vec<Block>> {
        // This would require implementing proper result parsing
        // For now, return empty vector
        Ok(vec![])
    }
}
