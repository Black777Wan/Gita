use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub id: String,
    pub content: Option<String>,
    pub parent_id: Option<String>,
    pub order: i32,
    pub is_page: bool,
    pub page_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub audio_timestamp: Option<AudioTimestamp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBlockRequest {
    pub content: Option<String>,
    pub parent_id: Option<String>,
    pub order: i32,
    pub is_page: bool,
    pub page_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioMeta {
    pub recording_id: String,
    pub timestamp: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioRecording {
    pub id: String,
    pub page_id: String,
    pub file_path: String,
    pub duration_seconds: Option<i32>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioTimestamp {
    pub id: i32,
    pub block_id: String,
    pub recording_id: String,
    pub timestamp_seconds: i32,
    pub recording: Option<AudioRecording>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
    pub device_type: String, // "input" or "output"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingState {
    pub is_recording: bool,
    pub recording_id: Option<String>,
    pub page_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
}

