use sqlx::PgPool;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::models::*;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        // Get database URL from environment variable or use default
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://gita_user:12345@localhost/gita_db".to_string());
        
        let pool = PgPool::connect(&database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Database { pool })
    }

    pub async fn get_daily_note(&self, date: &str) -> Result<Vec<Block>> {
        let page_title = format!("Daily Notes/{}", date);
        
        // First, try to get the daily note page
        if let Some(page) = self.get_page_by_title(&page_title).await? {
            // Get all children blocks
            let mut blocks = self.get_block_children(&page.id).await?;
            blocks.insert(0, page);
            Ok(blocks)
        } else {
            // Create the daily note page
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

    pub async fn create_block(
        &self,
        block_data: CreateBlockRequest,
        audio_meta: Option<AudioMeta>,
    ) -> Result<Block> {
        let id = Uuid::new_v4();
        let parent_id = match block_data.parent_id {
            Some(pid) => Some(Uuid::parse_str(&pid)?),
            None => None,
        };
        let now = Utc::now();

        let block = sqlx::query!(
            r#"
            INSERT INTO blocks (id, content, parent_id, "order", is_page, page_title, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, content, parent_id, "order", is_page, page_title, created_at, updated_at
            "#,
            id,
            block_data.content,
            parent_id,
            block_data.order,
            block_data.is_page,
            block_data.page_title,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        let id_str = id.to_string();

        // If audio metadata is provided, create the timestamp link
        if let Some(audio_meta) = audio_meta {
            self.create_audio_timestamp(&id_str, &audio_meta.recording_id, audio_meta.timestamp).await?;
        }

        // Fetch the block with audio timestamp if it exists
        self.get_block_with_audio_timestamp(&id_str).await
    }

    pub async fn update_block_content(&self, block_id: &str, content: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE blocks 
            SET content = $1, updated_at = $2 
            WHERE id = $3
            "#,
            content,
            Utc::now(),
            block_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_page_by_title(&self, title: &str) -> Result<Option<Block>> {
        let block = sqlx::query_as!(
            Block,
            r#"
            SELECT id, content, parent_id, "order", is_page, page_title, created_at, updated_at
            FROM blocks 
            WHERE page_title = $1 AND is_page = true
            "#,
            title
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(mut block) = block {
            block.audio_timestamp = self.get_block_audio_timestamp(&block.id).await?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    pub async fn get_block_children(&self, parent_id: &str) -> Result<Vec<Block>> {
        let blocks = sqlx::query_as!(
            Block,
            r#"
            SELECT id, content, parent_id, "order", is_page, page_title, created_at, updated_at
            FROM blocks 
            WHERE parent_id = $1 
            ORDER BY "order" ASC
            "#,
            parent_id
        )
        .fetch_all(&self.pool)
        .await?;

        // Fetch audio timestamps for all blocks
        let mut result = Vec::new();
        for mut block in blocks {
            block.audio_timestamp = self.get_block_audio_timestamp(&block.id).await?;
            result.push(block);
        }

        Ok(result)
    }

    pub async fn delete_block(&self, block_id: &str) -> Result<()> {
        sqlx::query!(
            "DELETE FROM blocks WHERE id = $1",
            block_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_audio_recording(
        &self,
        recording_id: &str,
        page_id: &str,
        file_path: &str,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO audio_recordings (id, page_id, file_path, recorded_at)
            VALUES ($1, $2, $3, $4)
            "#,
            recording_id,
            page_id,
            file_path,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_recording_duration(
        &self,
        recording_id: &str,
        duration_seconds: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE audio_recordings 
            SET duration_seconds = $1 
            WHERE id = $2
            "#,
            duration_seconds,
            recording_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_audio_timestamp(
        &self,
        block_id: &str,
        recording_id: &str,
        timestamp_seconds: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO audio_timestamps (block_id, recording_id, timestamp_seconds)
            VALUES ($1, $2, $3)
            ON CONFLICT (block_id, recording_id) 
            DO UPDATE SET timestamp_seconds = EXCLUDED.timestamp_seconds
            "#,
            block_id,
            recording_id,
            timestamp_seconds
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_block_audio_timestamp(&self, block_id: &str) -> Result<Option<AudioTimestamp>> {
        let row = sqlx::query!(
            r#"
            SELECT 
                at.id,
                at.block_id,
                at.recording_id,
                at.timestamp_seconds,
                ar.page_id,
                ar.file_path,
                ar.duration_seconds,
                ar.recorded_at
            FROM audio_timestamps at
            JOIN audio_recordings ar ON at.recording_id = ar.id
            WHERE at.block_id = $1
            "#,
            block_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let recording = AudioRecording {
                id: row.recording_id.clone(),
                page_id: row.page_id,
                file_path: row.file_path,
                duration_seconds: row.duration_seconds,
                recorded_at: row.recorded_at,
            };

            let timestamp = AudioTimestamp {
                id: row.id,
                block_id: row.block_id,
                recording_id: row.recording_id,
                timestamp_seconds: row.timestamp_seconds,
                recording: Some(recording),
            };

            Ok(Some(timestamp))
        } else {
            Ok(None)
        }
    }

    async fn get_block_with_audio_timestamp(&self, block_id: &str) -> Result<Block> {
        let mut block = sqlx::query_as!(
            Block,
            r#"
            SELECT id, content, parent_id, "order", is_page, page_title, created_at, updated_at
            FROM blocks 
            WHERE id = $1
            "#,
            block_id
        )
        .fetch_one(&self.pool)
        .await?;

        block.audio_timestamp = self.get_block_audio_timestamp(block_id).await?;
        Ok(block)
    }
}

