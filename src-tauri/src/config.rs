use std::env;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatomicConfig {
    pub db_uri: String,
    pub transactor_host: String,
    pub transactor_port: u16,
    pub database_name: String,
    pub datomic_lib_path: Option<PathBuf>,
    pub jvm_opts: Vec<String>,
    pub connection_timeout_ms: u64,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub recordings_dir: PathBuf,
    pub max_recording_duration_minutes: u32,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub datomic: DatomicConfig,
    pub audio: AudioConfig,
    pub log_level: String,
    pub data_dir: PathBuf,
}

impl Default for DatomicConfig {
    fn default() -> Self {
        Self {
            db_uri: "datomic:dev://localhost:8998/gita".to_string(),
            transactor_host: "localhost".to_string(),
            transactor_port: 8998,
            database_name: "gita".to_string(),
            datomic_lib_path: None,
            jvm_opts: vec![
                "-Xmx4g".to_string(),
                "-Xms1g".to_string(),
                "-XX:+UseG1GC".to_string(),
            ],
            connection_timeout_ms: 30000,
            retry_attempts: 3,
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            recordings_dir: PathBuf::from("recordings"),
            max_recording_duration_minutes: 120,
            sample_rate: 44100,
            channels: 2,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gita");
        
        let recordings_dir = data_dir.join("recordings");
        
        Self {
            datomic: DatomicConfig::default(),
            audio: AudioConfig {
                recordings_dir,
                ..AudioConfig::default()
            },
            log_level: "info".to_string(),
            data_dir,
        }
    }
}

impl AppConfig {
    /// Load configuration from file or environment variables
    pub fn load() -> Result<Self> {
        let mut config = Self::default();
        
        // Try to load from config file
        if let Ok(config_content) = std::fs::read_to_string("gita-config.toml") {
            config = toml::from_str(&config_content)
                .map_err(|e| anyhow!("Failed to parse config file: {}", e))?;
        }
        
        // Override with environment variables
        if let Ok(db_uri) = env::var("GITA_DB_URI") {
            config.datomic.db_uri = db_uri;
        }
        
        if let Ok(host) = env::var("GITA_DB_HOST") {
            config.datomic.transactor_host = host;
        }
        
        if let Ok(port_str) = env::var("GITA_DB_PORT") {
            config.datomic.transactor_port = port_str.parse()
                .map_err(|e| anyhow!("Invalid port number: {}", e))?;
        }
        
        if let Ok(lib_path) = env::var("DATOMIC_LIB_PATH") {
            config.datomic.datomic_lib_path = Some(PathBuf::from(lib_path));
        }
        
        if let Ok(log_level) = env::var("GITA_LOG_LEVEL") {
            config.log_level = log_level;
        }
        
        // Auto-detect Datomic installation if not specified
        if config.datomic.datomic_lib_path.is_none() {
            config.datomic.datomic_lib_path = Self::detect_datomic_installation();
        }
        
        // Ensure directories exist
        std::fs::create_dir_all(&config.data_dir)?;
        std::fs::create_dir_all(&config.audio.recordings_dir)?;
        
        Ok(config)
    }
    
    /// Auto-detect Datomic installation path
    fn detect_datomic_installation() -> Option<PathBuf> {
        // Common installation paths
        let common_paths = vec![
            PathBuf::from("C:\\Users\\yashd\\datomic-pro-1.0.7387\\lib"),
            PathBuf::from("C:\\datomic-pro\\lib"),
            PathBuf::from("/usr/local/datomic-pro/lib"),
            PathBuf::from("/opt/datomic-pro/lib"),
            PathBuf::from(env::var("HOME").unwrap_or_default()).join("datomic-pro/lib"),
        ];
        
        // Check if DATOMIC_HOME is set
        if let Ok(datomic_home) = env::var("DATOMIC_HOME") {
            let lib_path = PathBuf::from(datomic_home).join("lib");
            if lib_path.exists() {
                return Some(lib_path);
            }
        }
        
        // Check common paths
        for path in common_paths {
            if path.exists() && path.join("datomic-pro.jar").exists() {
                return Some(path);
            }
        }
        
        None
    }
    
    /// Generate JVM classpath for Datomic
    pub fn get_datomic_classpath(&self) -> Result<String> {
        let lib_path = self.datomic.datomic_lib_path.as_ref()
            .ok_or_else(|| anyhow!("Datomic lib path not configured. Please set DATOMIC_LIB_PATH environment variable or ensure Datomic is installed in a standard location."))?;
        
        if !lib_path.exists() {
            return Err(anyhow!("Datomic lib path does not exist: {}", lib_path.display()));
        }
        
        // Build classpath with all JAR files
        let mut classpath = Vec::new();
        
        for entry in std::fs::read_dir(lib_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                classpath.push(path.to_string_lossy().to_string());
            }
        }
        
        if classpath.is_empty() {
            return Err(anyhow!("No JAR files found in Datomic lib path: {}", lib_path.display()));
        }
        
        Ok(classpath.join(";"))
    }
    
    /// Save current configuration to file
    pub fn save(&self) -> Result<()> {
        let config_content = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
        
        std::fs::write("gita-config.toml", config_content)
            .map_err(|e| anyhow!("Failed to write config file: {}", e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.datomic.transactor_port, 8998);
        assert_eq!(config.datomic.database_name, "gita");
        assert_eq!(config.audio.sample_rate, 44100);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        
        assert_eq!(config.datomic.transactor_port, deserialized.datomic.transactor_port);
        assert_eq!(config.audio.sample_rate, deserialized.audio.sample_rate);
    }
}
