//! Async SQLite access layer using SQLx.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use uuid::Uuid;

use crate::models::*;

/* -------------------------------------------------------------------- */

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        // Create data directory if it doesn't exist
        let data_dir = std::env::current_dir()?.join("data");
        std::fs::create_dir_all(&data_dir)?;
        
        let db_path = data_dir.join("gita.db");
        let url = format!("sqlite://{}", db_path.to_string_lossy());

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    /* ------------------------- daily notes --------------------------- */

    pub async fn get_daily_note(&self, date: &str) -> Result<Vec<Block>> {
        let title = format!("Daily Notes/{date}");

        if let Some(page) = self.get_page_by_title(&title).await? {
            let mut children = self.get_block_children(&page.id).await?;
            children.insert(0, page);
            Ok(children)
        } else {
            let req = CreateBlockRequest {
                content: None,
                parent_id: None,
                order: 0,
                is_page: true,
                page_title: Some(title),
            };
            let page = self.create_block(req, None).await?;
            Ok(vec![page])
        }
    }

    /* ----------------------------- CRUD ----------------------------- */

    pub async fn create_block(
        &self,
        req: CreateBlockRequest,
        audio: Option<AudioMeta>,
    ) -> Result<Block> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO blocks (id, content, parent_id, "order", is_page, page_title, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            id,
            req.content,
            req.parent_id,
            req.order,
            req.is_page,
            req.page_title,
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        if let Some(a) = audio {
            self.create_audio_timestamp(&id, &a.recording_id, a.timestamp)
                .await?;
        }

        self.get_block_with_audio_timestamp(&id).await
    }

    pub async fn update_block_content(&self, id: &str, content: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query!(
            r#"UPDATE blocks SET content=?1, updated_at=?2 WHERE id=?3"#,
            content,
            now,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_block(&self, id: &str) -> Result<()> {
        sqlx::query!(r#"DELETE FROM blocks WHERE id=?1"#, id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /* --------------------------- readers ---------------------------- */

    pub async fn get_page_by_title(&self, title: &str) -> Result<Option<Block>> {
        let mut page = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, content, parent_id, "order", is_page, page_title,
                   created_at, updated_at
            FROM blocks
            WHERE page_title=?1 AND is_page=1
            "#,
        )
        .bind(title)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref mut p) = page {
            p.audio_timestamp = self.get_block_audio_timestamp(&p.id).await?;
        }
        Ok(page)
    }

    pub async fn get_block_children(&self, parent_id: &str) -> Result<Vec<Block>> {
        let mut rows = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, content, parent_id, "order", is_page, page_title,
                   created_at, updated_at
            FROM blocks
            WHERE parent_id=?1
            ORDER BY "order"
            "#,
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;

        for r in &mut rows {
            r.audio_timestamp = self.get_block_audio_timestamp(&r.id).await?;
        }
        Ok(rows)
    }

    pub async fn get_pages(&self) -> Result<Vec<Block>> {
        let mut pages = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, content, parent_id, "order", is_page, page_title,
                   created_at, updated_at
            FROM blocks
            WHERE is_page=1
            ORDER BY page_title
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for page in &mut pages {
            page.audio_timestamp = self.get_block_audio_timestamp(&page.id).await?;
        }
        Ok(pages)
    }

    /* ------------------------ audio metadata ------------------------ */

    pub async fn create_audio_recording(
        &self,
        recording_id: &str,
        page_id: &str,
        path: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query!(
            r#"INSERT INTO audio_recordings (id,page_id,file_path,recorded_at)
               VALUES (?1,?2,?3,?4)"#,
            recording_id,
            page_id,
            path,
            now
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_recording_duration(
        &self,
        recording_id: &str,
        secs: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"UPDATE audio_recordings SET duration_seconds=?1 WHERE id=?2"#,
            secs,
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
        secs: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO audio_timestamps (block_id,recording_id,timestamp_seconds)
            VALUES (?1,?2,?3)
            ON CONFLICT (block_id,recording_id)
              DO UPDATE SET timestamp_seconds = excluded.timestamp_seconds
            "#,
            block_id,
            recording_id,
            secs
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_block_audio_timestamp(
        &self,
        block_id: &str,
    ) -> Result<Option<AudioTimestamp>> {
        let row = sqlx::query!(
            r#"
            SELECT at.id,
                   at.block_id,
                   at.recording_id,
                   at.timestamp_seconds,
                   ar.page_id,
                   ar.file_path,
                   ar.duration_seconds,
                   ar.recorded_at
            FROM audio_timestamps at
            JOIN audio_recordings ar ON at.recording_id = ar.id
            WHERE at.block_id=?1
            "#,
            block_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let recording_id = r.recording_id.clone();
            Ok(Some(AudioTimestamp {
                id: r.id.unwrap_or(0) as i32,
                block_id: r.block_id,
                recording_id: r.recording_id,
                timestamp_seconds: r.timestamp_seconds as i32,
                recording: Some(AudioRecording {
                    id: recording_id,
                    page_id: r.page_id,
                    file_path: r.file_path,
                    duration_seconds: r.duration_seconds.map(|d| d as i32),
                    recorded_at: r.recorded_at,
                }),
            }))
        } else {
            Ok(None)
        }
    }

    /* ------------------------- private helper ----------------------- */

    async fn get_block_with_audio_timestamp(&self, id: &str) -> Result<Block> {
        let mut blk = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, content, parent_id, "order", is_page, page_title,
                   created_at, updated_at
            FROM blocks WHERE id=?1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        blk.audio_timestamp = self.get_block_audio_timestamp(id).await?;
        Ok(blk)
    }
}
