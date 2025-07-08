@echo off
echo Starting Gita Development Environment...
echo.

REM Change to the project root directory
cd /d "%~dp0"

REM Check if Node.js is installed
where node >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Error: Node.js is not installed or not in PATH
    echo Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

REM Check if Rust/Cargo is installed
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo Error: Rust/Cargo is not installed or not in PATH
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

REM Install frontend dependencies if node_modules doesn't exist
if not exist "frontend\node_modules" (
    echo Installing frontend dependencies...
    cd frontend
    npm install
    cd ..
    echo.
)

echo Starting Tauri development server...
echo This will automatically start both the frontend React server and the Tauri backend.
echo.

cargo tauri dev

pause
