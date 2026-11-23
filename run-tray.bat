@echo off
setlocal
cd /d "%~dp0"
if not exist "target\release\opencode-go-tray.exe" (
  echo Building OpenCode Go Meter...
  cargo build --release
  if errorlevel 1 (
    echo Build failed.
    pause
    exit /b 1
  )
)
start "" "target\release\opencode-go-tray.exe"
