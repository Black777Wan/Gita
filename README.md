# Gita - Research & Audio Note-Taking App

Gita is a desktop-first, note-taking application inspired by Roam Research with deep audio integration capabilities. Built with Tauri (Rust backend) and React (TypeScript frontend).

## Features

- **Block-based Note-Taking**: Create nested, bullet-point style notes similar to Roam Research
- **Bi-directional Linking**: Link pages together using `[[Page Title]]` syntax
- **Audio Recording**: Record microphone and system audio while taking notes
- **Audio Timestamping**: Every block created during recording is automatically timestamped
- **Audio Playback**: Click play buttons next to blocks to jump to the exact moment in the recording
- **Daily Notes**: Automatic daily note creation and navigation
- **Auto-save**: Real-time saving with debounced auto-save functionality
- **Cross-platform**: Runs on Windows, macOS, and Linux

## Architecture

### Backend (Rust/Tauri)
- **Audio Engine**: Cross-platform audio recording using `cpal` and `hound`
- **Database**: SQLite integration with `sqlx` for local data storage
- **File Management**: Local audio file storage in temporary directories
- **API**: Tauri commands for frontend-backend communication
- **Migration System**: Automatic database schema migrations on startup

### Frontend (React/TypeScript)
- **Block Editor**: Rich text editing with block-based structure and auto-save
- **Audio Controls**: Recording interface and playback controls
- **State Management**: Zustand for application state with pending save tracking
- **UI Components**: Modern, responsive interface with Tailwind CSS
- **Real-time Sync**: Automatic synchronization between frontend state and database

### Database Schema (SQLite)
- **blocks**: Core note/page storage with hierarchical structure
- **audio_recordings**: Audio file metadata and duration tracking
- **audio_timestamps**: Links blocks to specific audio timestamps

## Project Structure

```
gita/
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── main.rs           # Main Tauri application with command handlers
│   │   ├── database.rs       # SQLite database operations and migrations
│   │   ├── audio_engine.rs   # Cross-platform audio recording engine
│   │   └── models.rs         # Data structures and type definitions
│   ├── migrations/           # SQLite database schema migrations
│   │   └── 20240101000000_initial_sqlite.sql
│   ├── data/                 # Local SQLite database storage
│   │   └── gita.db          # SQLite database file (auto-created)
│   ├── gen/                  # Generated Tauri schemas and capabilities
│   ├── icons/               # Application icons for different platforms
│   ├── target/              # Rust build artifacts
│   ├── Cargo.toml           # Rust dependencies and project configuration
│   ├── tauri.conf.json      # Tauri application configuration
│   └── build.rs             # Build script
├── frontend/                 # React frontend
│   ├── src/
│   │   ├── components/       # React components
│   │   │   ├── AudioControls.tsx    # Audio recording controls
│   │   │   ├── BlockEditor.tsx      # Individual block editing with auto-save
│   │   │   ├── MainEditor.tsx       # Main editing interface
│   │   │   └── Sidebar.tsx          # Navigation sidebar
│   │   ├── store/
│   │   │   └── appStore.ts   # Zustand state management with pending saves
│   │   ├── App.tsx          # Main application component
│   │   ├── App.css          # Application styles
│   │   ├── index.tsx        # React entry point
│   │   └── index.css        # Global styles
│   ├── public/              # Static assets
│   │   └── index.html       # HTML template
│   ├── build/               # Production build output
│   ├── package.json         # Node.js dependencies
│   ├── tailwind.config.js   # Tailwind CSS configuration
│   ├── postcss.config.js    # PostCSS configuration
│   └── tsconfig.json        # TypeScript configuration
├── .gitignore              # Git ignore rules (includes database exclusion)
├── dev.bat                 # Windows development script
├── dev.ps1                 # PowerShell development script
├── package.json            # Root package.json for workspace
└── README.md
```

## Prerequisites

### System Dependencies
- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Node.js**: Version 16 or higher
- **System Libraries** (Linux):
  ```bash
  sudo apt update
  sudo apt install -y build-essential libwebkit2gtk-4.0-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev pkg-config
  ```

### Database Setup
No manual database setup required! The application uses SQLite and automatically:
- Creates the database file on first run
- Runs migrations to set up the schema
- Stores data in the system's temporary directory

**Database Location:**
- **Windows**: `%TEMP%\gita\data\gita.db`
- **Linux/macOS**: `/tmp/gita/data/gita.db` or `/home/ubuntu/gita/data/gita.db`

## Installation & Development

### 1. Clone the Repository
```bash
git clone https://github.com/Black777Wan/Gita.git
cd Gita
```

### 2. Install Tauri CLI
```bash
cargo install tauri-cli
```

### 3. Install Frontend Dependencies
```bash
cd frontend
npm install
cd ..
```

### 4. Run in Development Mode
```bash
# From the root directory
cargo tauri dev
```

**Alternative Development Scripts:**
- **Windows**: `dev.bat`
- **PowerShell**: `.\dev.ps1`

This will:
- Start the React development server on port 3000 (or next available port)
- Launch the Tauri application window
- Enable hot reloading for both frontend and backend changes
- Automatically create and migrate the SQLite database

### 5. Build for Production
```bash
cargo tauri build
```

The built application will be available in `src-tauri/target/release/`.

## Usage

### Basic Note-Taking
1. **Daily Notes**: Click on dates in the sidebar to create/open daily notes
2. **Create Blocks**: Press Enter to create new bullet points
3. **Edit Blocks**: Click on any block to edit its content
4. **Auto-save**: Content is automatically saved 200ms after you stop typing
5. **Manual Save**: Press Enter or click away to force immediate save
6. **Link Pages**: Use `[[Page Name]]` syntax to create links between pages
7. **Delete Blocks**: Clear all content and press Enter to delete a block

### Audio Recording
1. **Start Recording**: Click the microphone button in the header
2. **Take Notes**: Create blocks while recording - they'll be automatically timestamped
3. **Stop Recording**: Click the stop button to end the recording
4. **Playback**: Click the play button next to any timestamped block
5. **Audio Indicators**: Look for volume icons next to blocks with audio

### Navigation
- **Sidebar**: Browse daily notes (last 7 days) and custom pages
- **Search**: Use the search box to find specific content (coming soon)
- **Page Links**: Click on `[[Page Name]]` links to navigate between pages
- **Create Pages**: Type `[[New Page Name]]` and click the link to create new pages

### Data Persistence
- **Real-time Saving**: All changes are saved automatically with visual indicators
- **Pending Save Tracking**: The app tracks unsaved changes and prevents data loss
- **Navigation Safety**: The app waits for pending saves before switching pages
- **Session Recovery**: Data persists between app sessions in the local SQLite database

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

### Current Implementation Status
- ✅ **Core Note-Taking**: Block-based editing with auto-save
- ✅ **Database**: SQLite with automatic migrations
- ✅ **Audio Recording**: Cross-platform microphone recording
- ✅ **Audio Timestamping**: Automatic block-to-audio linking
- ✅ **Page Linking**: Bi-directional `[[Page Name]]` syntax
- ✅ **Daily Notes**: Automatic daily note generation
- ✅ **Real-time Sync**: Frontend-backend state synchronization
- 🚧 **Search**: Basic search interface (functionality pending)
- 🚧 **System Audio**: Platform-specific system audio capture
- 📋 **Export/Import**: Data export and backup features

### Audio Implementation
The audio engine uses `cpal` for cross-platform audio capture. Current implementation:
- **Microphone Recording**: ✅ Fully implemented
- **System Audio Loopback**: 🚧 Platform-specific implementation needed
  - **Windows**: WASAPI loopback mode (planned)
  - **macOS**: Core Audio with aggregate devices (planned)
  - **Linux**: PulseAudio/ALSA loopback (planned)

### Database Architecture
- **SQLite**: Local database stored in system temp directory
- **Auto-migration**: Schema updates applied automatically on startup
- **Schema**: Blocks with hierarchical parent-child relationships
- **Audio Metadata**: Recording files linked to specific blocks with timestamps

### State Management Architecture
The frontend uses Zustand with several key features:
- **Pending Save Tracking**: Monitors blocks with unsaved changes
- **Auto-save Debouncing**: 200ms delay to batch rapid changes
- **Navigation Safety**: Waits for saves before page transitions
- **Error Handling**: Graceful fallback for offline/Tauri unavailable scenarios

### Known Issues & Solutions
1. **Data Loss on Navigation**: ✅ Fixed with pending save tracking
2. **Database File Permissions**: ✅ Resolved by using temp directory
3. **Port Conflicts**: App automatically selects available ports
4. **Unused Import Warnings**: Minor Rust warnings (cosmetic only)

## Technical Details

### Backend Architecture (src-tauri/)
```rust
// Key Tauri Commands
get_daily_note(date)           // Load/create daily notes
create_block(data, audio)      // Create new blocks with optional audio
update_block_content(id, text) // Update block content with auto-save
get_page_by_title(title)       // Load existing pages
get_block_children(parent_id)  // Load hierarchical block structure
start_recording(page_id)       // Begin audio recording
stop_recording(recording_id)   // End recording and save duration
```

### Frontend Architecture (frontend/src/)
```typescript
// State Management (Zustand)
interface AppState {
  blocks: Block[]              // Current page blocks
  currentPage?: Block          // Active page
  pendingSaves: Set<string>    // Tracks unsaved blocks
  audioState: AudioState       // Recording status
  // ... actions for CRUD operations
}

// Auto-save with debouncing
debouncedSave(content, 200ms)  // Save after 200ms of no typing
handleBlur()                   // Force save on focus loss
waitForPendingSaves()          // Block navigation until saves complete
```

### Database Schema
```sql
-- Core block storage with hierarchical structure
CREATE TABLE blocks (
    id TEXT PRIMARY KEY,
    content TEXT,
    parent_id TEXT,              -- Self-referencing for hierarchy
    "order" INTEGER,             -- Sort order within parent
    is_page BOOLEAN,             -- Distinguishes pages from blocks
    page_title TEXT,             -- Page title for navigation
    created_at TEXT,
    updated_at TEXT
);

-- Audio recording metadata
CREATE TABLE audio_recordings (
    id TEXT PRIMARY KEY,
    page_id TEXT,                -- Links to parent page
    file_path TEXT,              -- Local WAV file path
    duration_seconds INTEGER,
    recorded_at TEXT
);

-- Links blocks to audio timestamps
CREATE TABLE audio_timestamps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    block_id TEXT,               -- Links to specific block
    recording_id TEXT,           -- Links to audio file
    timestamp_seconds INTEGER,   -- Playback position
    UNIQUE (block_id, recording_id)
);
```

## Contributing

### Development Workflow
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes following the existing code style
4. Test thoroughly on your target platform(s)
5. Update documentation if needed
6. Submit a pull request with a clear description

### Code Guidelines
- **Rust**: Follow standard Rust conventions, use `cargo fmt` and `cargo clippy`
- **TypeScript**: Use proper typing, follow React hooks patterns
- **Database**: Create migration files for schema changes
- **Testing**: Test both development and production builds

### Platform Testing
Please test changes on:
- Windows 10/11
- macOS (if available)
- Linux (Ubuntu/Debian preferred)

## Troubleshooting

### Common Issues

**App won't start / Database errors:**
```bash
# Clear database and restart
rm -rf %TEMP%\gita  # Windows
rm -rf /tmp/gita     # Linux/macOS
cargo tauri dev
```

**Port already in use:**
- The frontend will automatically prompt to use another port
- Answer 'Y' to continue on a different port

**Audio recording not working:**
- Check microphone permissions in system settings
- Ensure default microphone is properly configured
- Try restarting the app

**Build failures:**
```bash
# Clean and rebuild
cd frontend && npm install
cd .. && cargo clean
cargo tauri build
```

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- Inspired by [Roam Research](https://roamresearch.com/) for block-based note-taking
- Built with [Tauri](https://tauri.app/) for the desktop application framework
- Uses [SQLx](https://github.com/launchbadge/sqlx) for database operations
- Audio engine powered by [cpal](https://github.com/RustAudio/cpal)
- Frontend built with React, TypeScript, and Tailwind CSS

---

**Version**: 0.1.0  
**Last Updated**: January 2025  
**Minimum Supported Platforms**: Windows 10+, macOS 10.15+, Linux (GTK 3.0+)

