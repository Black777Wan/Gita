#!/bin/bash

# Gita Production Build Script
# This script creates a production-ready build of the Gita application

set -e

echo "🚀 Starting Gita Production Build"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Rust is not installed. Please install Rust from https://rustup.rs/${NC}"
    exit 1
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js is not installed. Please install Node.js from https://nodejs.org/${NC}"
    exit 1
fi

# Check if Datomic is configured
if [ -z "$DATOMIC_LIB_PATH" ] && [ ! -d "C:\\Users\\yashd\\datomic-pro-1.0.7387\\lib" ]; then
    echo -e "${YELLOW}⚠️  Datomic not found. Please set DATOMIC_LIB_PATH environment variable${NC}"
    echo -e "${YELLOW}   or install Datomic in a standard location${NC}"
fi

echo -e "${GREEN}✅ Prerequisites check completed${NC}"

# Clean previous builds
echo -e "${YELLOW}🧹 Cleaning previous builds...${NC}"
rm -rf src-tauri/target/release
rm -rf frontend/build
rm -rf dist

# Build frontend
echo -e "${YELLOW}🏗️  Building frontend...${NC}"
cd frontend
npm install
npm run build
cd ..

# Build Tauri application
echo -e "${YELLOW}🦀 Building Tauri application...${NC}"
cd src-tauri
cargo build --release
cd ..

# Run tests
echo -e "${YELLOW}🧪 Running tests...${NC}"
cd src-tauri
cargo test
cd ..

# Create distribution package
echo -e "${YELLOW}📦 Creating distribution package...${NC}"
mkdir -p dist

# Copy built application
if [ -f "src-tauri/target/release/gita.exe" ]; then
    cp src-tauri/target/release/gita.exe dist/
elif [ -f "src-tauri/target/release/gita" ]; then
    cp src-tauri/target/release/gita dist/
else
    echo -e "${RED}❌ Built application not found${NC}"
    exit 1
fi

# Copy necessary files
cp README.md dist/
cp LICENSE dist/ 2>/dev/null || echo "No LICENSE file found"
cp gita-config.toml dist/ 2>/dev/null || echo "No config file found"

# Create installation script
cat > dist/install.sh << 'EOF'
#!/bin/bash
echo "📦 Installing Gita..."

# Create application directory
mkdir -p ~/.local/share/gita
mkdir -p ~/.local/bin

# Copy application
cp gita ~/.local/bin/
chmod +x ~/.local/bin/gita

# Copy configuration
cp gita-config.toml ~/.local/share/gita/ 2>/dev/null || echo "No config file to copy"

# Create desktop entry (Linux)
if command -v xdg-desktop-menu &> /dev/null; then
    cat > ~/.local/share/applications/gita.desktop << 'DESKTOP_EOF'
[Desktop Entry]
Name=Gita
Comment=Research & Audio Note-Taking App
Exec=gita
Icon=gita
Type=Application
Categories=Office;Education;
DESKTOP_EOF
    xdg-desktop-menu install ~/.local/share/applications/gita.desktop
fi

echo "✅ Gita installed successfully!"
echo "📋 To run Gita, use: gita"
echo "🔧 Configuration file: ~/.local/share/gita/gita-config.toml"
EOF

chmod +x dist/install.sh

# Create Windows installation script
cat > dist/install.bat << 'EOF'
@echo off
echo Installing Gita...

REM Create application directory
mkdir "%USERPROFILE%\AppData\Local\Gita" 2>nul

REM Copy application
copy gita.exe "%USERPROFILE%\AppData\Local\Gita\"
copy gita-config.toml "%USERPROFILE%\AppData\Local\Gita\" 2>nul

REM Add to PATH (requires admin privileges)
echo Adding Gita to PATH...
setx PATH "%PATH%;%USERPROFILE%\AppData\Local\Gita"

echo Gita installed successfully!
echo To run Gita, use: gita
echo Configuration file: %USERPROFILE%\AppData\Local\Gita\gita-config.toml
pause
EOF

# Create setup instructions
cat > dist/SETUP.md << 'EOF'
# Gita Setup Instructions

## Prerequisites

### 1. Datomic Pro
- Download and install Datomic Pro from https://my.datomic.com/
- Set the `DATOMIC_LIB_PATH` environment variable to point to your Datomic lib directory
- Example: `export DATOMIC_LIB_PATH=/path/to/datomic-pro/lib`

### 2. Java Runtime
- Ensure Java 8 or later is installed
- Verify with: `java -version`

### 3. System Libraries (Linux only)
```bash
sudo apt update
sudo apt install -y build-essential libwebkit2gtk-4.0-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev pkg-config
```

## Installation

### Linux/macOS
```bash
chmod +x install.sh
./install.sh
```

### Windows
```cmd
install.bat
```

## Configuration

Edit the configuration file at:
- Linux/macOS: `~/.local/share/gita/gita-config.toml`
- Windows: `%USERPROFILE%\AppData\Local\Gita\gita-config.toml`

### Key Configuration Options

```toml
[datomic]
db_uri = "datomic:dev://localhost:8998/gita"
transactor_host = "localhost"
transactor_port = 8998
database_name = "gita"
datomic_lib_path = "/path/to/datomic-pro/lib"
connection_timeout_ms = 30000
retry_attempts = 3

[audio]
recordings_dir = "recordings"
max_recording_duration_minutes = 120
sample_rate = 44100
channels = 2

log_level = "info"
```

## Running Datomic Transactor

Before starting Gita, ensure the Datomic transactor is running:

```bash
# Navigate to your Datomic installation
cd /path/to/datomic-pro

# Start the transactor
bin/transactor -Xms1g -Xmx2g config/samples/dev-transactor-template.properties
```

## Starting Gita

```bash
gita
```

## Troubleshooting

### "JVM initialization failed"
- Ensure Java is installed and in PATH
- Verify `DATOMIC_LIB_PATH` is set correctly
- Check that all Datomic JAR files are present

### "Connection error"
- Ensure Datomic transactor is running
- Check the connection URI in config
- Verify firewall settings

### "Schema error"
- The application will automatically create the database schema
- If issues persist, check transactor logs

For more help, see the troubleshooting section in README.md
EOF

echo -e "${GREEN}✅ Build completed successfully!${NC}"
echo -e "${GREEN}📦 Distribution package created in: dist/${NC}"
echo -e "${GREEN}🚀 To install: cd dist && ./install.sh (or install.bat on Windows)${NC}"

# Create checksum file
echo -e "${YELLOW}🔐 Creating checksums...${NC}"
cd dist
find . -type f -exec sha256sum {} \; > checksums.txt
cd ..

echo -e "${GREEN}🎉 Gita production build completed successfully!${NC}"
echo -e "${GREEN}📋 Next steps:${NC}"
echo -e "${GREEN}   1. Test the application: cd dist && ./gita${NC}"
echo -e "${GREEN}   2. Install: ./install.sh${NC}"
echo -e "${GREEN}   3. Start Datomic transactor${NC}"
echo -e "${GREEN}   4. Run Gita application${NC}"
