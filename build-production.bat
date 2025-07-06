@echo off
REM Gita Production Build Script for Windows
REM This script creates a production-ready build of the Gita application

setlocal enabledelayedexpansion

echo 🚀 Starting Gita Production Build

REM Check prerequisites
echo 📋 Checking prerequisites...

REM Check if Rust is installed
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Rust is not installed. Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Check if Node.js is installed
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Node.js is not installed. Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

REM Check if Datomic is configured
if not defined DATOMIC_LIB_PATH (
    if not exist "C:\Users\yashd\datomic-pro-1.0.7387\lib" (
        echo ⚠️  Datomic not found. Please set DATOMIC_LIB_PATH environment variable
        echo    or install Datomic in a standard location
    )
)

echo ✅ Prerequisites check completed

REM Clean previous builds
echo 🧹 Cleaning previous builds...
if exist "src-tauri\target\release" rmdir /s /q "src-tauri\target\release"
if exist "frontend\build" rmdir /s /q "frontend\build"
if exist "dist" rmdir /s /q "dist"

REM Build frontend
echo 🏗️  Building frontend...
cd frontend
call npm install
call npm run build
cd ..

REM Build Tauri application
echo 🦀 Building Tauri application...
cd src-tauri
call cargo build --release
cd ..

REM Run tests
echo 🧪 Running tests...
cd src-tauri
call cargo test
cd ..

REM Create distribution package
echo 📦 Creating distribution package...
mkdir dist

REM Copy built application
if exist "src-tauri\target\release\gita.exe" (
    copy "src-tauri\target\release\gita.exe" "dist\"
) else (
    echo ❌ Built application not found
    pause
    exit /b 1
)

REM Copy necessary files
copy README.md dist\
copy LICENSE dist\ 2>nul || echo No LICENSE file found
copy gita-config.toml dist\ 2>nul || echo No config file found

REM Create installation script
echo Creating installation script...
(
echo @echo off
echo echo Installing Gita...
echo.
echo REM Create application directory
echo mkdir "%%USERPROFILE%%\AppData\Local\Gita" 2^>nul
echo.
echo REM Copy application
echo copy gita.exe "%%USERPROFILE%%\AppData\Local\Gita\"
echo copy gita-config.toml "%%USERPROFILE%%\AppData\Local\Gita\" 2^>nul
echo.
echo REM Add to PATH
echo echo Adding Gita to PATH...
echo setx PATH "%%PATH%%;%%USERPROFILE%%\AppData\Local\Gita"
echo.
echo echo Gita installed successfully!
echo echo To run Gita, use: gita
echo echo Configuration file: %%USERPROFILE%%\AppData\Local\Gita\gita-config.toml
echo pause
) > dist\install.bat

REM Create setup instructions
(
echo # Gita Setup Instructions
echo.
echo ## Prerequisites
echo.
echo ### 1. Datomic Pro
echo - Download and install Datomic Pro from https://my.datomic.com/
echo - Set the `DATOMIC_LIB_PATH` environment variable to point to your Datomic lib directory
echo - Example: `set DATOMIC_LIB_PATH=C:\path\to\datomic-pro\lib`
echo.
echo ### 2. Java Runtime
echo - Ensure Java 8 or later is installed
echo - Verify with: `java -version`
echo.
echo ## Installation
echo.
echo ### Windows
echo ```cmd
echo install.bat
echo ```
echo.
echo ## Configuration
echo.
echo Edit the configuration file at:
echo - Windows: `%%USERPROFILE%%\AppData\Local\Gita\gita-config.toml`
echo.
echo ### Key Configuration Options
echo.
echo ```toml
echo [datomic]
echo db_uri = "datomic:dev://localhost:8998/gita"
echo transactor_host = "localhost"
echo transactor_port = 8998
echo database_name = "gita"
echo datomic_lib_path = "C:\\path\\to\\datomic-pro\\lib"
echo connection_timeout_ms = 30000
echo retry_attempts = 3
echo.
echo [audio]
echo recordings_dir = "recordings"
echo max_recording_duration_minutes = 120
echo sample_rate = 44100
echo channels = 2
echo.
echo log_level = "info"
echo ```
echo.
echo ## Running Datomic Transactor
echo.
echo Before starting Gita, ensure the Datomic transactor is running:
echo.
echo ```cmd
echo cd C:\path\to\datomic-pro
echo bin\transactor -Xms1g -Xmx2g config\samples\dev-transactor-template.properties
echo ```
echo.
echo ## Starting Gita
echo.
echo ```cmd
echo gita
echo ```
echo.
echo ## Troubleshooting
echo.
echo ### "JVM initialization failed"
echo - Ensure Java is installed and in PATH
echo - Verify `DATOMIC_LIB_PATH` is set correctly
echo - Check that all Datomic JAR files are present
echo.
echo ### "Connection error"
echo - Ensure Datomic transactor is running
echo - Check the connection URI in config
echo - Verify firewall settings
echo.
echo ### "Schema error"
echo - The application will automatically create the database schema
echo - If issues persist, check transactor logs
echo.
echo For more help, see the troubleshooting section in README.md
) > dist\SETUP.md

echo ✅ Build completed successfully!
echo 📦 Distribution package created in: dist\
echo 🚀 To install: cd dist && install.bat

echo 🎉 Gita production build completed successfully!
echo 📋 Next steps:
echo    1. Test the application: cd dist && gita.exe
echo    2. Install: install.bat
echo    3. Start Datomic transactor
echo    4. Run Gita application

pause
