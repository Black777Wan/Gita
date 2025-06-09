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
- **Database**: PostgreSQL integration with `sqlx`
- **File Management**: Local audio file storage and management
- **API**: Tauri commands for frontend-backend communication

### Frontend (React/TypeScript)
- **Block Editor**: Rich text editing with block-based structure
- **Audio Controls**: Recording interface and playback controls
- **State Management**: Zustand for application state
- **UI Components**: Modern, responsive interface with Tailwind CSS

### Database Schema
- **blocks**: Core note/page storage with hierarchical structure
- **audio_recordings**: Audio file metadata and duration tracking
- **audio_timestamps**: Links blocks to specific audio timestamps

## Project Structure

```
gita/
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── main.rs           # Main Tauri application
│   │   ├── database.rs       # PostgreSQL operations
│   │   ├── audio_engine.rs   # Audio recording engine
│   │   └── models.rs         # Data structures
│   ├── migrations/           # Database migrations
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
- **PostgreSQL**: Local PostgreSQL server
- **System Libraries** (Linux):
  ```bash
  sudo apt update
  sudo apt install -y build-essential libwebkit2gtk-4.0-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev pkg-config
  ```

### Database Setup
1. Install and start PostgreSQL
2. Create database:
   ```sql
   CREATE DATABASE gita_db;
   CREATE USER gita_user WITH PASSWORD 'your_password';
   GRANT ALL PRIVILEGES ON DATABASE gita_db TO gita_user;
   ```
3. Update database URL in `src-tauri/src/database.rs`

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

### Database Migrations
Database schema changes are managed through SQL migration files in `src-tauri/migrations/`. The application automatically runs migrations on startup.

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

