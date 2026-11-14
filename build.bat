@echo off
setlocal
cd /d "%~dp0"
set BACKEND=%~dp0..\Runecord-backend

pushd "%BACKEND%"
cargo build --release -p runecord-server
if errorlevel 1 (popd & exit /b 1)
popd

cargo build --release -p runecord --features embedded-server
if errorlevel 1 exit /b 1

if not exist "%~dp0dist\assets" mkdir "%~dp0dist\assets"
copy /y "%~dp0target\release\runecord.exe" "%~dp0dist\Runecord.exe" >nul
copy /y "%BACKEND%\target\release\runecord-server.exe" "%~dp0dist\runecord-server.exe" >nul
copy /y "%~dp0assets\icon.png" "%~dp0dist\assets\icon.png" >nul

echo.
echo Double-click %~dp0dist\Runecord.exe
echo The backend is embedded; its data lives in %%APPDATA%%\Runecord\server
echo A standalone server copy is in %~dp0dist\runecord-server.exe
