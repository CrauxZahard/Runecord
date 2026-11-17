@echo off
setlocal
cd /d "%~dp0"
set PROFILE=%~1
if "%PROFILE%"=="" set PROFILE=1

rem Development build: one process, debug profile, demo accounts seeded by the
rem embedded server. run.bat 2 opens a second profile (maya/runecord).
cargo build -p runecord --features embedded-server || exit /b 1
start "" "target\debug\runecord.exe" --profile %PROFILE%
