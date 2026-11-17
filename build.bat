@echo off
setlocal
cd /d "%~dp0"

set FAILED=0

if not exist "%~dp0dist\assets" mkdir "%~dp0dist\assets"

rem The download page and the portable notes ship next to the artifacts.
copy /y "%~dp0download\index.html" "%~dp0dist\index.html" >nul
copy /y "%~dp0packaging\README.txt" "%~dp0dist\README.txt" >nul
copy /y "%~dp0LICENSE" "%~dp0dist\LICENSE" >nul
copy /y "%~dp0assets\icon.png" "%~dp0dist\assets\icon.png" >nul
copy /y "%~dp0assets\icon.ico" "%~dp0dist\assets\icon.ico" >nul

rem Release binary, signature and per-user installer. The script builds the
rem client with the embedded server, so this is the only Rust build a release
rem needs; the standalone server lives in the backend project.
call powershell -ExecutionPolicy Bypass -File "%~dp0scripts\build-installer.ps1"
if errorlevel 1 (
    echo warning: the Windows installer failed
    set FAILED=1
)
echo [1/2] Windows  dist\Runecord-Setup.exe

if /i "%~1"=="--desktop-only" goto :packaged

call powershell -ExecutionPolicy Bypass -File "%~dp0scripts\build-linux.ps1"
if errorlevel 1 (
    echo warning: the Linux build failed
    set FAILED=1
)
echo [2/2] Linux    dist\Runecord-linux.tar.gz

:packaged
echo.
echo Download page: %~dp0dist\index.html
echo Data lives in %%APPDATA%%\Runecord on Windows and ~/.local/share/runecord on Linux
echo.

rem The target folder is the build cache: keeping it is what makes the next
rem build recompile only what changed instead of the whole dependency tree.
rem Pass --clean when disk space matters more than build speed.
if /i "%~1"=="--clean" (
    cargo clean
    if errorlevel 1 (
        echo warning: could not clean the build tree
        exit /b 0
    )
    echo Cleaned the build tree.
)
exit /b %FAILED%
