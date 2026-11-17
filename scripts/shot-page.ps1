# Screenshots a local page at an exact CSS viewport.
#   scripts\shot-page.ps1 <url> <width> <height> <out.png> [dpr]
param(
    [Parameter(Mandatory = $true)][string]$Url,
    [Parameter(Mandatory = $true)][int]$Width,
    [Parameter(Mandatory = $true)][int]$Height,
    [Parameter(Mandatory = $true)][string]$Out,
    [double]$Dpr = 1,
    [string]$Eval = ""
)
$ErrorActionPreference = "Stop"
$edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
if (-not (Test-Path $edge)) {
    $edge = "C:\Program Files\Microsoft\Edge\Application\msedge.exe"
}
if (-not (Test-Path $edge)) { throw "Microsoft Edge is required." }

$profile = Join-Path $env:TEMP "runecord-edge-shot"
if (Test-Path $profile) { Remove-Item -Recurse -Force $profile -ErrorAction SilentlyContinue }
$browser = Start-Process -FilePath $edge -PassThru -ArgumentList @(
    "--headless=new",
    "--remote-debugging-port=9222",
    "--user-data-dir=$profile",
    "--no-first-run",
    "--no-default-browser-check",
    "--enable-unsafe-webgpu",
    "--disable-gpu-sandbox",
    "about:blank"
)
try {
    for ($i = 0; $i -lt 40; $i++) {
        try {
            Invoke-RestMethod -Uri "http://127.0.0.1:9222/json/version" -TimeoutSec 2 | Out-Null
            break
        } catch {
            Start-Sleep -Milliseconds 250
        }
    }
    node (Join-Path $PSScriptRoot "page-shot.mjs") $Url $Width $Height $Out $Dpr $Eval
}
finally {
    Get-CimInstance Win32_Process -Filter "Name='msedge.exe'" |
        Where-Object { $_.CommandLine -like "*runecord-edge-shot*" } |
        ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
}
