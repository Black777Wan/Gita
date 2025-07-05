use reqwest::Client;
use anyhow::{Result, anyhow};
use serde_json::{json, Value};
use crate::models::*;
use crate::datomic_schema::gita_schema;

const DATOMIC_API_URL: &str = "http://localhost:8998/api";
const DB_NAME: &str = "gita";

pub struct DatomicClient {
    client: Client,
    uri: String,
}

impl DatomicClient {
    pub async fn new() -> Result<Self> {
        let client = Client::new();
        let uri = format!("{}/{}", DATOMIC_API_URL, DB_NAME);
        let datomic_client = DatomicClient { client, uri };

        datomic_client.ensure_schema().await?;

        Ok(datomic_client)
    }

    async fn transact(&self, tx_data: Value) -> Result<Value> {
        let response = self.client
            .post(&format!("{}/", self.uri))
            .header("Content-Type", "application/edn")
            .body(format!("{{:tx-data {}}}", tx_data))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error_body = response.text().await?;
            Err(anyhow!("Datomic transaction failed: {}", error_body))
        }
    }

    async fn query(&self, query: &str, args: Value) -> Result<Value> {
        let response = self.client
            .post(&format!("{}/", self.uri))
            .header("Content-Type", "application/edn")
            .body(format!("{{:query '{}' :args {}}}", query, args))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error_body = response.text().await?;
            Err(anyhow!("Datomic query failed: {}", error_body))
        }
    }

    pub async fn ensure_schema(&self) -> Result<()> {
        // Check if schema is already present by querying for one of its attributes
        let check_query = r#"
            [:find ?e .
             :where [?e :db/ident :block/content]]
        "#;
        
        let result = self.query(check_query, json!([])).await;

        // If the query fails or returns no results, the schema is likely not present.
        if result.is_err() || result.unwrap().as_array().map_or(true, |r| r.is_empty()) {
            println!("Schema not found, attempting to transact it...");
            let schema_tx = gita_schema();
            let schema_value: Value = serde_json::from_str(&schema_tx)?;
            self.transact(schema_value).await?;
            println!("Schema transaction successful.");
        } else {
            println!("Schema already present.");
        }

        Ok(())
    }

    pub async fn create_block(
        &self,
        block_data: CreateBlockRequest,
        audio_meta: Option<AudioMeta>,
    ) -> Result<Block> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let temp_block_id = "new-block";

        let mut tx_data = vec![
            json!([":db/add", temp_block_id, ":block/id", block_id]),
            json!([":db/add", temp_block_id, ":block/is_page", block_data.is_page]),
            json!([":db/add", temp_block_id, ":block/order", block_data.order]),
            json!([":db/add", temp_block_id, ":block/created_at", now]),
            json!([":db/add", temp_block_id, ":block/updated_at", now]),
        ];

        if let Some(content) = block_data.content {
            tx_data.push(json!([":db/add", temp_block_id, ":block/content", content]));
        }

        if let Some(page_title) = block_data.page_title {
            tx_data.push(json!([":db/add", temp_block_id, ":block/page_title", page_title]));
        }

        if let Some(parent_id) = block_data.parent_id {
            tx_data.push(json!([":db/add", temp_block_id, ":block/parent", [":block/id", parent_id]]));
        }

        if let Some(audio_meta) = &audio_meta {
            let temp_ts_id = "new-timestamp";
            tx_data.push(json!([":db/add", temp_ts_id, ":timestamp/block", temp_block_id]));
            tx_data.push(json!([":db/add", temp_ts_id, ":timestamp/recording_id", audio_meta.recording_id]));
            tx_data.push(json!([":db/add", temp_ts_id, ":timestamp/timestamp_ms", audio_meta.timestamp]));
        }

        let _tx_result = self.transact(json!(tx_data)).await?;

        // Now that the block is created, we can fetch it to return the full Block struct
        self.get_block(&block_id).await
    }

    fn value_to_block(value: &Value) -> Result<Block> {
        Ok(Block {
            id: value[0].as_str().unwrap_or_default().to_string(),
            content: value[1].as_str().map(String::from),
            parent_id: value[2].as_str().map(String::from),
            order: value[3].as_i64().unwrap_or(0) as i32,
            is_page: value[4].as_bool().unwrap_or(false),
            page_title: value[5].as_str().map(String::from),
            created_at: chrono::DateTime::parse_from_rfc3339(value[6].as_str().unwrap_or_default())
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(value[7].as_str().unwrap_or_default())
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
            audio_timestamp: None, // This will be populated separately
        })
    }

    pub async fn get_block(&self, block_id: &str) -> Result<Block> {
        let query = r#"
            [:find ?id ?content ?parent_id ?order ?is_page ?page_title ?created_at ?updated_at
             :in $ ?id
             :where [?b :block/id ?id]
                    [(get-else $ ?b :block/content "") ?content]
                    [(get-else $ ?b :block/parent "") ?p]
                    [(get-else $ ?p :block/id "") ?parent_id]
                    [(get-else $ ?b :block/order 0) ?order]
                    [(get-else $ ?b :block/is_page false) ?is_page]
                    [(get-else $ ?b :block/page_title "") ?page_title]
                    [(get-else $ ?b :block/created_at "") ?created_at]
                    [(get-else $ ?b :block/updated_at "") ?updated_at]]
        "#;
        let args = json!([[block_id]]);
        let result = self.query(query, args).await?;
        let first_result = result.get(0).ok_or_else(|| anyhow!("Block not found"))?;
        let mut block = Self::value_to_block(first_result)?;
        block.audio_timestamp = self.get_block_audio_timestamp(&block.id).await?;
        Ok(block)
    }

    pub async fn get_page_by_title(&self, title: &str) -> Result<Option<Block>> {
        let query = r#"
            [:find ?id .
             :in $ ?title
             :where [?b :block/page_title ?title]
                    [?b :block/id ?id]]
        "#;
        let args = json!([[title]]);
        let result = self.query(query, args).await?;
        if let Some(id) = result.get(0).and_then(|v| v.as_str()) {
            Ok(Some(self.get_block(id).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_block_children(&self, parent_id: &str) -> Result<Vec<Block>> {
        let query = r#"
            [:find ?child_id
             :in $ ?parent_id
             :where [?p :block/id ?parent_id]
                    [?c :block/parent ?p]
                    [?c :block/id ?child_id]]
        "#;
        let args = json!([[parent_id]]);
        let result = self.query(query, args).await?;
        let mut children = Vec::new();
        if let Some(child_ids) = result.as_array() {
            for child_id in child_ids {
                if let Some(id) = child_id.as_str() {
                    children.push(self.get_block(id).await?);
                }
            }
        }
        children.sort_by_key(|b| b.order);
        Ok(children)
    }

    pub async fn get_block_audio_timestamp(&self, block_id: &str) -> Result<Option<AudioTimestamp>> {
        let query = r#"
            [:find ?recording_id ?timestamp_ms .
             :in $ ?block_id
             :where [?b :block/id ?block_id]
                    [?t :timestamp/block ?b]
                    [?t :timestamp/recording_id ?recording_id]
                    [?t :timestamp/timestamp_ms ?timestamp_ms]]
        "#;
        let args = json!([[block_id]]);
        let result = self.query(query, args).await?;
        if let Some(values) = result.get(0).and_then(|v| v.as_array()) {
            let recording_id = values[0].as_str().unwrap_or_default().to_string();
            let timestamp_seconds = values[1].as_i64().unwrap_or(0) as i32;
            Ok(Some(AudioTimestamp {
                block_id: block_id.to_string(),
                recording_id,
                timestamp_seconds,
                recording: None, // TODO: Implement fetching recording details
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn update_block_content(&self, block_id: &str, content: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let tx_data = json!([
            [":db/add", [":block/id", block_id], ":block/content", content],
            [":db/add", [":block/id", block_id], ":block/updated_at", now]
        ]);
        self.transact(tx_data).await?;
        Ok(())
    }

    pub async fn delete_block(&self, block_id: &str) -> Result<()> {
        let tx_data = json!([
            [":db/retractEntity", [":block/id", block_id]]
        ]);
        self.transact(tx_data).await?;
        Ok(())
    }

    pub async fn create_audio_recording(
        &self,
        recording_id: &str,
        page_id: &str,
        file_path: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let temp_audio_id = "new-audio";
        let tx_data = json!([
            [":db/add", temp_audio_id, ":audio/id", recording_id],
            [":db/add", temp_audio_id, ":audio/page", [":block/id", page_id]],
            [":db/add", temp_audio_id, ":audio/path", file_path],
            [":db/add", temp_audio_id, ":audio/created_at", now]
        ]);
        self.transact(tx_data).await?;
        Ok(())
    }

    pub async fn update_recording_duration(
        &self,
        recording_id: &str,
        duration_seconds: i32,
    ) -> Result<()> {
        let tx_data = json!([
            [":db/add", [":audio/id", recording_id], ":audio/duration", duration_seconds]
        ]);
        self.transact(tx_data).await?;
        Ok(())
    }

    pub async fn create_audio_timestamp(
        &self,
        block_id: &str,
        recording_id: &str,
        timestamp_seconds: i32,
    ) -> Result<()> {
        let temp_ts_id = "new-timestamp";
        let tx_data = json!([
            [":db/add", temp_ts_id, ":timestamp/block", [":block/id", block_id]],
            [":db/add", temp_ts_id, ":timestamp/recording_id", recording_id],
            [":db/add", temp_ts_id, ":timestamp/timestamp_ms", timestamp_seconds]
        ]);
        self.transact(tx_data).await?;
        Ok(())
    }
    
    pub async fn get_daily_note(&self, date: &str) -> Result<Vec<Block>> {
        let page_title = format!("Daily Notes/{}", date);
        if let Some(page) = self.get_page_by_title(&page_title).await? {
            let mut blocks = self.get_block_children(&page.id).await?;
            blocks.insert(0, page);
            Ok(blocks)
        } else {
            let create_request = CreateBlockRequest {
                content: None,
                parent_id: None,
                order: 0,
                is_page: true,
                page_title: Some(page_title),
            };
            let page = self.create_block(create_request, None).await?;
            Ok(vec![page])
        }
    }
}