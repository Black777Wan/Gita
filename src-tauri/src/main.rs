// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod audio_engine;
mod models;
mod datomic_schema;

use std::sync::{Arc, Mutex};
use tauri::Manager;
use audio_engine::AudioEngine;
use models::*;
use database::DatomicClient;

// Tauri commands for database operations
#[tauri::command]
async fn get_daily_note(
    date: String,
    db: tauri::State<'_, DatomicClient>,
) -> Result<Vec<Block>, String> {
    db.get_daily_note(&date).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_block(
    block_data: CreateBlockRequest,
    audio_meta: Option<AudioMeta>,
    db: tauri::State<'_, DatomicClient>,
) -> Result<Block, String> {
    let block = db.create_block(block_data, audio_meta.clone()).await.map_err(|e| e.to_string())?;
    if let Some(audio_meta) = audio_meta {
        db.create_audio_timestamp(&block.id, &audio_meta.recording_id, audio_meta.timestamp).await.map_err(|e| e.to_string())?;
    }
    Ok(block)
}

#[tauri::command]
async fn update_block_content(
    block_id: String,
    content: String,
    db: tauri::State<'_, DatomicClient>,
) -> Result<(), String> {
    db.update_block_content(&block_id, &content).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_page_by_title(
    title: String,
    db: tauri::State<'_, DatomicClient>,
) -> Result<Option<Block>, String> {
    db.get_page_by_title(&title).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_block_children(
    parent_id: String,
    db: tauri::State<'_, DatomicClient>,
) -> Result<Vec<Block>, String> {
    db.get_block_children(&parent_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_block(
    block_id: String,
    db: tauri::State<'_, DatomicClient>,
) -> Result<(), String> {
    db.delete_block(&block_id).await.map_err(|e| e.to_string())
}

// Audio commands
#[tauri::command]
async fn start_recording(
    page_id: String,
    audio_engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
    db: tauri::State<'_, DatomicClient>,
) -> Result<String, String> {
    let recording_id = uuid::Uuid::new_v4().to_string();
    let file_path = format!("/home/ubuntu/gita/audio/{}.wav", recording_id);
    
    // Create audio recording entry in database
    db.create_audio_recording(&recording_id, &page_id, &file_path).await
        .map_err(|e| e.to_string())?;
    
    // Start audio capture
    let engine = audio_engine.lock().unwrap();
    engine.start_recording(&file_path).map_err(|e| e.to_string())?;
    
    Ok(recording_id)
}

#[tauri::command]
async fn stop_recording(
    recording_id: String,
    audio_engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
    db: tauri::State<'_, DatomicClient>,
) -> Result<(), String> {
    // Stop audio capture and get duration
    let duration = {
        let engine = audio_engine.lock().unwrap();
        engine.stop_recording().map_err(|e| e.to_string())?
    }; // Mutex guard is dropped here
    
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
    db: tauri::State<'_, DatomicClient>,
) -> Result<Option<AudioTimestamp>, String> {
    db.get_block_audio_timestamp(&block_id).await.map_err(|e| e.to_string())
}

fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize Datomic client
            let datomic_client = tauri::async_runtime::block_on(async {
                DatomicClient::new().await.expect("Failed to initialize Datomic client")
            });
            
            // Initialize audio engine
            let audio_engine = Arc::new(Mutex::new(
                AudioEngine::new().expect("Failed to initialize audio engine")
            ));
            
            // Create audio directory
            std::fs::create_dir_all("/home/ubuntu/gita/audio")
                .expect("Failed to create audio directory");
            
            app.manage(datomic_client);
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

