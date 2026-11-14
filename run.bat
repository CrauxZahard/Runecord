@echo off
setlocal
cd /d "%~dp0"
set PROFILE=%~1
if "%PROFILE%"=="" set PROFILE=1
set BACKEND=%~dp0..\Runecord-backend

rem Local builds run in debug so the launcher always shows the current code.
pushd "%BACKEND%"
cargo build -p runecord-server || (popd & exit /b 1)
popd
cargo build -p runecord || exit /b 1

start "" "target\debug\runecord.exe" --profile %PROFILE%
