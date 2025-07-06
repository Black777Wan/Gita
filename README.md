# Gita - Research & Audio Note-Taking App

Gita is a desktop-first, note-taking application inspired by Roam Research with deep audio integration capabilities. Built with Tauri (Rust backend) and React (TypeScript frontend).

## Features

- **Block-based Note-Taking**: Create nested, bullet-point style notes similar to Roam Research
- **Bi-directional Linking**: Link pages together using `[[Page Title]]` syntax
- **Audio Recording**: Record microphone and system audio while taking notes
- **Audio Timestamping**: Every block created during recording is automatically timestamped
- **Audio Playback**: Click play buttons next to blocks to jump to the exact moment in the recording
- **Daily Notes**: Automatic daily note creation and navigation
- **Cross-platform**: Runs on Windows, macOS, and Linux

## Architecture

### Backend (Rust/Tauri)
- **Audio Engine**: Cross-platform audio recording using `cpal` and `hound`
- **Database**: Datomic Peer API for flexible, graph-based data storage with JNI integration
- **File Management**: Local audio file storage and management
- **API**: Tauri commands for frontend-backend communication
- **Configuration**: TOML-based configuration with environment variable support
- **Error Handling**: Comprehensive error handling and retry logic
- **Logging**: Structured logging with tracing

### Frontend (React/TypeScript)
- **Block Editor**: Rich text editing with block-based structure
- **Audio Controls**: Recording interface and playback controls
- **State Management**: Zustand for application state
- **UI Components**: Modern, responsive interface with Tailwind CSS

### Database Schema

The application uses Datomic, a flexible, graph-based database. The schema is defined in `src-tauri/src/datomic_schema.rs` and includes the following main entities:

- **Blocks**: The core of the note-taking system. Each block has a unique ID, content, order, and can be part of a page or nested under another block.
- **Audio Recordings**: Stores metadata about recorded audio files, including the file path and duration.
- **Audio Timestamps**: Links a specific block to a point in time within an audio recording.

## Project Structure

```
gita/
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── main.rs           # Main Tauri application
│   │   ├── database_peer_complete.rs # Datomic Peer API client
│   │   ├── audio_engine.rs   # Audio recording engine
│   │   ├── models.rs         # Data structures
│   │   ├── datomic_schema.rs # Datomic schema definition
│   │   ├── config.rs         # Configuration management
│   │   ├── errors.rs         # Error handling
│   │   └── tests.rs          # Test suites
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── frontend/                 # React frontend
│   ├── src/
│   │   ├── components/       # React components
│   │   ├── store/           # Zustand state management
│   │   ├── App.tsx          # Main application component
│   │   └── index.tsx        # React entry point
│   ├── public/              # Static assets
│   └── package.json         # Node.js dependencies
└── README.md
```

## Prerequisites

### System Dependencies
- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Node.js**: Version 16 or higher
- **Datomic**: A local Datomic dev-pro server is required. You can run it using Docker.
- **System Libraries** (Linux):
  ```bash
  sudo apt update
  sudo apt install -y build-essential libwebkit2gtk-4.0-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev pkg-config
  ```

### Database Setup

**Important Note**: This application now uses Datomic's Peer API, which requires Datomic Pro and a running transactor. The application directly connects to the transactor using JNI (Java Native Interface).

#### Prerequisites
1. **Datomic Pro License** - Required for production use
2. **Java Runtime** - Java 8 or later (Java 17 recommended)
3. **Datomic Installation** - Download from https://my.datomic.com/

#### Setup Steps

1. **Install Datomic Pro**
   ```powershell
   # Download and extract Datomic Pro
   # Set DATOMIC_LIB_PATH environment variable
   $env:DATOMIC_LIB_PATH = "C:\Users\yashd\datomic-pro-1.0.7387\lib"
   ```

2. **Start the Datomic transactor**
   ```powershell
   cd C:\Users\yashd\datomic-pro-1.0.7387
   bin\transactor -Xms1g -Xmx2g config\samples\dev-transactor-template.properties
   ```

3. **Configure Gita**
   
   Create `gita-config.toml` in your data directory:
   ```toml
   [datomic]
   db_uri = "datomic:dev://localhost:8998/gita"
   transactor_host = "localhost"
   transactor_port = 8998
   database_name = "gita"
   datomic_lib_path = "C:\\Users\\yashd\\datomic-pro-1.0.7387\\lib"
   connection_timeout_ms = 30000
   retry_attempts = 3
   jvm_opts = ["-Xmx4g", "-Xms1g", "-XX:+UseG1GC"]

   [audio]
   recordings_dir = "recordings"
   max_recording_duration_minutes = 120
   sample_rate = 44100
   channels = 2

   log_level = "info"
   ```

4. **(Optional) Start the Datomic Console**
   ```powershell
   cd C:\Users\yashd\datomic-pro-1.0.7387
   bin\console -p 8080 dev datomic:dev://localhost:8998/
   ```
   Access at `http://localhost:8080/browse`

### Verifying Datomic Setup
After starting the Datomic transactor, you can verify that the database is set up correctly:

1. **Check the Transactor**: The transactor should be running on port 8998. You can verify this by running `netstat -an | findstr "8998"` in PowerShell.
2. **Check Application Health**: Once Gita is running, it includes a health check endpoint to verify database connectivity.
3. **Access the Web Console** (if started): Open `http://localhost:8080/browse` in your browser to access the Datomic console.
4. **Connect to the Database** in the console:
   - **Storage**: Select `dev`
   - **DB Name**: Enter `gita`
5. **Explore**: Click "Connect". If successful, you'll be able to browse the schema and data.

The application will automatically:
- Create the `gita` database if it doesn't exist
- Transact the schema on first run
- Perform health checks and retry failed connections
- Log all operations for troubleshooting

## Installation & Development

### 1. Install Tauri CLI
```bash
cargo install tauri-cli
```

### 2. Install Frontend Dependencies
```bash
cd frontend
npm install
```

### 3. Run in Development Mode
```bash
# From the root directory
cargo tauri dev
```

This will:
- Start the React development server
- Launch the Tauri application
- Enable hot reloading for both frontend and backend

### 4. Build for Production
```bash
cargo tauri build
```

## Usage

### Basic Note-Taking
1. **Daily Notes**: Click on dates in the sidebar to create/open daily notes
2. **Create Blocks**: Press Enter to create new bullet points
3. **Edit Blocks**: Click on any block to edit its content
4. **Link Pages**: Use `[[Page Name]]` syntax to create links between pages

### Audio Recording
1. **Start Recording**: Click the microphone button in the header
2. **Take Notes**: Create blocks while recording - they'll be automatically timestamped
3. **Stop Recording**: Click the stop button to end the recording
4. **Playback**: Click the play button next to any timestamped block

### Navigation
- **Sidebar**: Browse daily notes and pages
- **Search**: Use the search box to find specific content
- **Page Links**: Click on `[[Page Name]]` links to navigate

## Audio Features

### Recording Capabilities
- **Microphone Input**: Records from default or selected microphone
- **System Audio**: Captures system audio output (implementation varies by platform)
- **Simultaneous Capture**: Records both sources and mixes them into a single file
- **Real-time Timestamping**: Links each block to the exact moment it was created

### Playback Features
- **Instant Playback**: Click play buttons to jump to specific timestamps
- **Audio Indicators**: Visual indicators show which blocks have audio
- **Timestamp Display**: Shows the exact time in the recording for each block

## Development Notes

### Audio Implementation
The audio engine uses `cpal` for cross-platform audio capture. System audio loopback recording requires platform-specific implementations:
- **Windows**: WASAPI loopback mode
- **macOS**: Core Audio with aggregate devices
- **Linux**: PulseAudio/ALSA loopback

### Datomic Schema

The Datomic schema is defined in `src-tauri/src/datomic_schema.rs`. The application ensures that the schema is present in the database on startup. If the schema is not found, it is automatically transacted.

### State Management
The frontend uses Zustand for state management, providing a simple and efficient way to manage application state across components.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- Inspired by [Roam Research](https://roamresearch.com/)
- Built on [free-roam](https://github.com/cofinley/free-roam) open-source foundation
- Powered by [Tauri](https://tauri.app/) for desktop application framework
