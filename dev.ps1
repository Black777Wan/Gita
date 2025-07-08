# Gita Development Startup Script
Write-Host "Starting Gita Development Environment..." -ForegroundColor Green
Write-Host ""

# Change to the script's directory
Set-Location $PSScriptRoot

# Check if Node.js is installed
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Node.js is not installed or not in PATH" -ForegroundColor Red
    Write-Host "Please install Node.js from https://nodejs.org/" -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

# Check if Rust/Cargo is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust/Cargo is not installed or not in PATH" -ForegroundColor Red
    Write-Host "Please install Rust from https://rustup.rs/" -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

# Install frontend dependencies if node_modules doesn't exist
if (-not (Test-Path "frontend\node_modules")) {
    Write-Host "Installing frontend dependencies..." -ForegroundColor Yellow
    Set-Location frontend
    npm install
    Set-Location ..
    Write-Host ""
}

Write-Host "Starting Tauri development server..." -ForegroundColor Green
Write-Host "This will automatically start both the frontend React server and the Tauri backend." -ForegroundColor Cyan
Write-Host ""

# Start the development server
cargo tauri dev

Read-Host "Press Enter to exit"
