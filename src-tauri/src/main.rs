// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod audio_engine;
mod models;

use std::sync::{Arc, Mutex};
use tauri::Manager;
use database::Database;
use audio_engine::AudioEngine;
use models::*;

// Tauri commands for database operations
#[tauri::command]
async fn get_daily_note(
    date: String,
    db: tauri::State<'_, Database>,
) -> Result<Vec<Block>, String> {
    db.get_daily_note(&date).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_block(
    block_data: CreateBlockRequest,
    audio_meta: Option<AudioMeta>,
    db: tauri::State<'_, Database>,
) -> Result<Block, String> {
    db.create_block(block_data, audio_meta).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_block_content(
    block_id: String,
    content: String,
    db: tauri::State<'_, Database>,
) -> Result<(), String> {
    db.update_block_content(&block_id, &content).await.map_err(|e| e.to_string())
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
    db.get_block_children(&parent_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_block(
    block_id: String,
    db: tauri::State<'_, Database>,
) -> Result<(), String> {
    db.delete_block(&block_id).await.map_err(|e| e.to_string())
}

// Audio commands
#[tauri::command]
async fn start_recording(
    page_id: String,
    audio_engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
    db: tauri::State<'_, Database>,
) -> Result<String, String> {
    let recording_id = uuid::Uuid::new_v4().to_string();
    let file_path = format!("/home/ubuntu/gita/audio/{}.wav", recording_id);
    
    // Create audio recording entry in database
    db.create_audio_recording(&recording_id, &page_id, &file_path).await
        .map_err(|e| e.to_string())?;
    
    // Start audio capture
    let mut engine = audio_engine.lock().unwrap();
    engine.start_recording(&file_path).map_err(|e| e.to_string())?;
    
    Ok(recording_id)
}

#[tauri::command]
async fn stop_recording(
    recording_id: String,
    audio_engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
    db: tauri::State<'_, Database>,
) -> Result<(), String> {
    // Stop audio capture
    let mut engine = audio_engine.lock().unwrap();
    let duration = engine.stop_recording().map_err(|e| e.to_string())?;
    
    // Update recording duration in database
    db.update_recording_duration(&recording_id, duration).await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn get_audio_devices(
    audio_engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
) -> Result<Vec<AudioDevice>, String> {
    let engine = audio_engine.lock().unwrap();
    engine.get_audio_devices().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_block_audio_timestamp(
    block_id: String,
    db: tauri::State<'_, Database>,
) -> Result<Option<AudioTimestamp>, String> {
    db.get_block_audio_timestamp(&block_id).await.map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let database = tauri::async_runtime::block_on(async {
                Database::new().await.expect("Failed to initialize database")
            });
            
            // Initialize audio engine
            let audio_engine = Arc::new(Mutex::new(
                AudioEngine::new().expect("Failed to initialize audio engine")
            ));
            
            // Create audio directory
            std::fs::create_dir_all("/home/ubuntu/gita/audio")
                .expect("Failed to create audio directory");
            
            app.manage(database);
            app.manage(audio_engine);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_daily_note,
            create_block,
            update_block_content,
            get_page_by_title,
            get_block_children,
            delete_block,
            start_recording,
            stop_recording,
            get_audio_devices,
            get_block_audio_timestamp
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

