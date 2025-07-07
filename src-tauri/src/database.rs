//! Async Postgres access layer using SQLx.

use anyhow::Result;
use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

use crate::models::*;

/* -------------------------------------------------------------------- */

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://gita_user:12345@localhost/gita_db".into());

        let pool = PgPoolOptions::new()
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
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO blocks (id,content,parent_id,"order",is_page,page_title,created_at,updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
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

    pub async fn update_block_content(&self, id: &Uuid, content: &str) -> Result<()> {
        sqlx::query!(
            r#"UPDATE blocks SET content=$1, updated_at=$2 WHERE id=$3"#,
            content,
            Utc::now(),
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_block(&self, id: &Uuid) -> Result<()> {
        sqlx::query!(r#"DELETE FROM blocks WHERE id=$1"#, id)
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
            WHERE page_title=$1 AND is_page=true
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

    pub async fn get_block_children(&self, parent_id: &Uuid) -> Result<Vec<Block>> {
        let mut rows = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, content, parent_id, "order", is_page, page_title,
                   created_at, updated_at
            FROM blocks
            WHERE parent_id=$1
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

    /* ------------------------ audio metadata ------------------------ */

    pub async fn create_audio_recording(
        &self,
        recording_id: &Uuid,
        page_id: &Uuid,
        path: &str,
    ) -> Result<()> {
        sqlx::query!(
            r#"INSERT INTO audio_recordings (id,page_id,file_path,recorded_at)
               VALUES ($1,$2,$3,$4)"#,
            recording_id,
            page_id,
            path,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_recording_duration(
        &self,
        recording_id: &Uuid,
        secs: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"UPDATE audio_recordings SET duration_seconds=$1 WHERE id=$2"#,
            secs,
            recording_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn create_audio_timestamp(
        &self,
        block_id: &Uuid,
        recording_id: &Uuid,
        secs: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO audio_timestamps (block_id,recording_id,timestamp_seconds)
            VALUES ($1,$2,$3)
            ON CONFLICT (block_id,recording_id)
              DO UPDATE SET timestamp_seconds = EXCLUDED.timestamp_seconds
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
        block_id: &Uuid,
    ) -> Result<Option<AudioTimestamp>> {
        let row = sqlx::query!(
            r#"
            SELECT at.id,
                   at.block_id,
                   at.recording_id,
                   at.timestamp_seconds,
                   ar.page_id             as "page_id!:Uuid",
                   ar.file_path,
                   ar.duration_seconds,
                   ar.recorded_at         as "recorded_at?:DateTime<Utc>"
            FROM audio_timestamps at
            JOIN audio_recordings ar ON at.recording_id = ar.id
            WHERE at.block_id=$1
            "#,
            block_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            Ok(Some(AudioTimestamp {
                id: r.id,
                block_id: r.block_id,
                recording_id: r.recording_id,
                timestamp_seconds: r.timestamp_seconds,
                recording: Some(AudioRecording {
                    id: r.recording_id,
                    page_id: r.page_id,
                    file_path: r.file_path,
                    duration_seconds: r.duration_seconds,
                    recorded_at: r.recorded_at,
                }),
            }))
        } else {
            Ok(None)
        }
    }

    /* ------------------------- private helper ----------------------- */

    async fn get_block_with_audio_timestamp(&self, id: &Uuid) -> Result<Block> {
        let mut blk = sqlx::query_as::<_, Block>(
            r#"
            SELECT id, content, parent_id, "order", is_page, page_title,
                   created_at, updated_at
            FROM blocks WHERE id=$1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        blk.audio_timestamp = self.get_block_audio_timestamp(id).await?;
        Ok(blk)
    }
}
