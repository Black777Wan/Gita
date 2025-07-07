//! Tauri entry‑point wiring HTTP‑like commands to the Database & AudioEngine.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio_engine;
mod database;
mod models;

use std::sync::{Arc, Mutex};

use audio_engine::AudioEngine;
use database::Database;
use models::*;

use tauri::Manager;
use uuid::Uuid;

/* -------------------- database‑backed commands -------------------- */

#[tauri::command]
async fn get_daily_note(date: String, db: tauri::State<'_, Database>) -> Result<Vec<Block>, String> {
    db.get_daily_note(&date).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_block(
    block: CreateBlockRequest,
    audio: Option<AudioMeta>,
    db: tauri::State<'_, Database>,
) -> Result<Block, String> {
    db.create_block(block, audio).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_block_content(
    block_id: String,
    content: String,
    db: tauri::State<'_, Database>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&block_id).map_err(|e| e.to_string())?;
    db.update_block_content(&id, &content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_page_by_title(
    title: String,
    db: tauri::State<'_, Database>,
) -> Result<Option<Block>, String> {
    db.get_page_by_title(&title).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_block_children(
    parent_id: String,
    db: tauri::State<'_, Database>,
) -> Result<Vec<Block>, String> {
    let id = Uuid::parse_str(&parent_id).map_err(|e| e.to_string())?;
    db.get_block_children(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_block(block_id: String, db: tauri::State<'_, Database>) -> Result<(), String> {
    let id = Uuid::parse_str(&block_id).map_err(|e| e.to_string())?;
    db.delete_block(&id).await.map_err(|e| e.to_string())
}

/* -------------------------- audio commands ------------------------- */

#[tauri::command]
async fn start_recording(
    page_id: String,
    engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
    db: tauri::State<'_, Database>,
) -> Result<String, String> {
    let page_uuid = Uuid::parse_str(&page_id).map_err(|e| e.to_string())?;
    let rec_id = Uuid::new_v4();
    let path = format!("/home/ubuntu/gita/audio/{rec_id}.wav");

    db.create_audio_recording(&rec_id, &page_uuid, &path)
        .await
        .map_err(|e| e.to_string())?;

    engine
        .lock()
        .unwrap()
        .start_recording(&path)
        .map_err(|e| e.to_string())?;

    Ok(rec_id.to_string())
}

#[tauri::command]
async fn stop_recording(
    recording_id: String,
    engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
    db: tauri::State<'_, Database>,
) -> Result<(), String> {
    let rec_uuid = Uuid::parse_str(&recording_id).map_err(|e| e.to_string())?;
    let secs = engine
        .lock()
        .unwrap()
        .stop_recording()
        .map_err(|e| e.to_string())?;

    db.update_recording_duration(&rec_uuid, secs)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_audio_devices(
    engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
) -> Result<Vec<AudioDevice>, String> {
    engine
        .lock()
        .unwrap()
        .get_audio_devices()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_block_audio_timestamp(
    block_id: String,
    db: tauri::State<'_, Database>,
) -> Result<Option<AudioTimestamp>, String> {
    let id = Uuid::parse_str(&block_id).map_err(|e| e.to_string())?;
    db.get_block_audio_timestamp(&id)
        .await
        .map_err(|e| e.to_string())
}

/* ------------------------------------------------------------------ */

fn main() {
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            /* database */
            let db = tauri::async_runtime::block_on(Database::new())
                .expect("DB init failed");
            app.manage(db);

            /* audio */
            std::fs::create_dir_all("/home/ubuntu/gita/audio").ok();
            let engine = Arc::new(Mutex::new(
                AudioEngine::new().expect("audio init failed"),
            ));
            app.manage(engine);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            /* db */
            get_daily_note,
            create_block,
            update_block_content,
            get_page_by_title,
            get_block_children,
            delete_block,
            /* audio */
            start_recording,
            stop_recording,
            get_audio_devices,
            get_block_audio_timestamp
        ])
        .run(tauri::generate_context!())
        .expect("tauri run error");
}

