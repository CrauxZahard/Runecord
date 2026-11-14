@echo off
setlocal
cd /d "%~dp0"
echo This deletes the backend database, uploads and saved sessions.
choice /M "Continue"
if errorlevel 2 exit /b 0
if exist "..\Runecord-backend\data" rmdir /s /q "..\Runecord-backend\data"
if exist "%APPDATA%\Runecord" rmdir /s /q "%APPDATA%\Runecord"
echo Done. The next launch seeds the demo server again.
