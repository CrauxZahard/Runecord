@echo off
setlocal
cd /d "%~dp0"
set BACKEND=%~dp0..\Runecord-backend

pushd "%BACKEND%"
cargo build --release -p runecord-server
if errorlevel 1 (popd & exit /b 1)
popd

cargo build --release -p runecord
if errorlevel 1 exit /b 1
echo.
echo Built %BACKEND%\target\release\runecord-server.exe
echo Built %~dp0target\release\runecord.exe
