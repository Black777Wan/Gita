// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database_peer_complete;
mod audio_engine;
mod models;
mod datomic_schema;
mod config;
mod errors;

#[cfg(test)]
mod tests;

extern crate tracing; // Removed #[macro_use]

use std::sync::{Arc, Mutex};
use tauri::Manager;
use tracing::{info, error, Level};
use tracing_subscriber;

use audio_engine::AudioEngine;
use models::*;
use database_peer_complete::DatomicPeerClient;
use config::AppConfig;
// Removed DatomicError, Result as they are not directly used in this file
// use errors::{DatomicError, Result};

// Tauri commands for database operations
#[tauri::command]
async fn get_daily_note(
    date: String,
    db: tauri::State<'_, DatomicPeerClient>,
) -> std::result::Result<Vec<Block>, String> {
    db.inner().get_daily_note(&date).await.map_err(|e| {
        error!("Failed to get daily note for {}: {}", date, e);
        e.to_string()
    })
}

#[tauri::command]
async fn create_block(
    block_data: CreateBlockRequest,
    audio_meta: Option<AudioMeta>,
    db: tauri::State<'_, DatomicPeerClient>,
) -> std::result::Result<Block, String> {
    db.inner().create_block(block_data, audio_meta).await.map_err(|e| {
        error!("Failed to create block: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn update_block_content(
    block_id: String,
    content: String,
    db: tauri::State<'_, DatomicPeerClient>,
) -> std::result::Result<(), String> {
    let mut updates = std::collections::HashMap::new();
    updates.insert("content".to_string(), serde_json::Value::String(content));
    db.inner().update_block(&block_id, updates).await.map_err(|e| {
        error!("Failed to update block {}: {}", block_id, e);
        e.to_string()
    })?;
    Ok(())
}

#[tauri::command]
async fn get_page_by_title(
    title: String,
    db: tauri::State<'_, DatomicPeerClient>,
) -> std::result::Result<Option<Block>, String> {
    db.inner().get_page_blocks(&title).await
        .map(|blocks| blocks.first().cloned())
        .map_err(|e| {
            error!("Failed to get page by title {}: {}", title, e);
            e.to_string()
        })
}

#[tauri::command]
async fn get_block_children(
    parent_id: String,
    db: tauri::State<'_, DatomicPeerClient>,
) -> std::result::Result<Vec<Block>, String> {
    db.inner().get_page_blocks(&parent_id).await.map_err(|e| {
        error!("Failed to get block children for {}: {}", parent_id, e);
        e.to_string()
    })
}

#[tauri::command]
async fn search_blocks(
    query: String,
    db: tauri::State<'_, DatomicPeerClient>,
) -> std::result::Result<Vec<Block>, String> {
    db.inner().search_blocks(&query).await.map_err(|e| {
        error!("Failed to search blocks for '{}': {}", query, e);
        e.to_string()
    })
}

#[tauri::command]
async fn delete_block(
    block_id: String,
    _db: tauri::State<'_, DatomicPeerClient>, // Prefixed with underscore
) -> std::result::Result<(), String> {
    // TODO: Implement delete_block in the peer client
    error!("Delete block not yet implemented for block_id: {}", block_id);
    Err("Delete block not yet implemented".to_string())
}

// Audio commands
#[tauri::command]
async fn start_recording(
    page_id: String,
    audio_engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
    _db: tauri::State<'_, DatomicPeerClient>, // Prefixed with underscore
) -> std::result::Result<String, String> {
    let recording_id = uuid::Uuid::new_v4().to_string();
    let file_path = format!("./audio/{}.wav", recording_id);
    
    // Create audio recording entry in database
    let _recording = AudioRecording { // Underscore to silence unused warning for now
        id: recording_id.clone(),
        page_id: page_id.clone(),
        file_path: file_path.clone(),
        duration_seconds: None,
        recorded_at: chrono::Utc::now(),
    };
    
    // TODO: Implement create_audio_recording in the peer client
    // db.inner().create_audio_recording(recording).await.map_err(|e| e.to_string())?;
    
    // Start audio capture
    let engine = audio_engine.lock().unwrap();
    engine.start_recording(&file_path).map_err(|e| e.to_string())?;
    
    Ok(recording_id)
}

#[tauri::command]
async fn stop_recording(
    recording_id: String,
    audio_engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
    _db: tauri::State<'_, DatomicPeerClient>, // Prefixed with underscore
) -> std::result::Result<(), String> {
    // Stop audio capture and get duration
    let _duration = { // Underscore to silence unused warning for now
        let engine = audio_engine.lock().unwrap();
        engine.stop_recording().map_err(|e| e.to_string())?
    }; // Mutex guard is dropped here
    
    // Update recording duration in database
    // TODO: Implement update_audio_recording in the peer client
    info!("Stopped recording: {}", recording_id);
    Ok(())
}

#[tauri::command]
async fn get_audio_devices(
    audio_engine: tauri::State<'_, Arc<Mutex<AudioEngine>>>,
) -> std::result::Result<Vec<AudioDevice>, String> {
    let engine = audio_engine.lock().unwrap();
    engine.get_audio_devices().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_block_audio_timestamp(
    block_id: String,
    _db: tauri::State<'_, DatomicPeerClient>, // Prefixed with underscore
) -> std::result::Result<Option<AudioTimestamp>, String> {
    // TODO: Implement get_block_audio_timestamp in the peer client
    info!("get_block_audio_timestamp called for: {}", block_id);
    Ok(None)
}

#[tauri::command]
async fn health_check(
    db: tauri::State<'_, DatomicPeerClient>,
) -> std::result::Result<bool, String> {
    db.inner().health_check().await.map_err(|e| {
        error!("Health check failed: {}", e);
        e.to_string()
    })
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    info!("Starting Gita application");
    
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            info!("Setting up Tauri application");
            
            // Load configuration
            let config = AppConfig::load()
                .expect("Failed to load application configuration");
            
            info!("Loaded configuration: {:?}", config);
            
            // Initialize Datomic peer client
            let datomic_client = tauri::async_runtime::block_on(async {
                match DatomicPeerClient::new(config.clone()).await {
                    Ok(client) => {
                        info!("Datomic peer client initialized successfully");
                        client
                    }
                    Err(e) => {
                        error!("Failed to initialize Datomic peer client: {}", e);
                        panic!("Cannot start application without database connection");
                    }
                }
            });
            
            // Initialize audio engine
            let audio_engine = Arc::new(Mutex::new(
                AudioEngine::new().expect("Failed to initialize audio engine")
            ));
            
            // Create necessary directories
            std::fs::create_dir_all(&config.audio.recordings_dir)
                .expect("Failed to create recordings directory");
            
            info!("Application setup completed successfully");
            
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
            search_blocks,
            delete_block,
            start_recording,
            stop_recording,
            get_audio_devices,
            get_block_audio_timestamp,
            health_check
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
    
    info!("Gita application shut down");
}

