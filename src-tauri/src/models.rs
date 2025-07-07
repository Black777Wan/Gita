//! Plain‑data structs shared across front‑end, database and audio layers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single row from the `blocks` table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Block {
    pub id: Uuid,
    pub content: Option<String>,
    pub parent_id: Option<Uuid>,

    #[sqlx(rename = "order")]
    pub order: i32,

    pub is_page: Option<bool>,
    pub page_title: Option<String>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,

    /// Populated on demand from `audio_timestamps` / `audio_recordings`.
    #[sqlx(skip)]
    pub audio_timestamp: Option<AudioTimestamp>,
}

/* -------------------------------------------------------------------- */

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBlockRequest {
    pub content: Option<String>,
    pub parent_id: Option<Uuid>,
    pub order: i32,
    pub is_page: bool,
    pub page_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioMeta {
    pub recording_id: Uuid,
    /// Seconds since the beginning of the recording.
    pub timestamp: i32,
}

/* ----------------------------- audio types --------------------------- */

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct AudioRecording {
    pub id: Uuid,
    pub page_id: Uuid,
    pub file_path: String,
    pub duration_seconds: Option<i32>,
    pub recorded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioTimestamp {
    pub id: i32,
    pub block_id: Uuid,
    pub recording_id: Uuid,
    pub timestamp_seconds: i32,
    pub recording: Option<AudioRecording>,
}

/* --------------------------- UI convenience -------------------------- */

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
    /// `"input"` or `"output"`
    pub device_type: String,
}
