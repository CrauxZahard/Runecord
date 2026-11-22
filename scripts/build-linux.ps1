# Linux release build
#
# Cross-compiles the desktop client for x86_64 Linux with zig as the linker,
# then packs the binary, icon and a readme into dist\Runecord-linux.tar.gz.
#
#   scripts\build-linux.ps1 [-SkipBuild]
param([switch]$SkipBuild)
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$zig = Get-ChildItem "D:\tools\zig" -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -like "zig-*" } |
    Select-Object -First 1
if (-not $zig) { throw "Install zig under D:\tools\zig first." }
if (-not (Get-Command cargo-zigbuild -ErrorAction SilentlyContinue)) {
    throw "Install cargo-zigbuild first: cargo install cargo-zigbuild"
}
$env:PATH = "$($zig.FullName);$env:PATH"

# Rune links libxkbcommon on Linux (keymaps for Wayland overlays). On Windows,
# zig's lld cannot open a library found through an absolute `-L` search path,
# so pass the vendored library as a direct linker input. The runtime uses the
# distribution's libxkbcommon.so.0; this copy is only for linking.
$library = Join-Path $root "vendor\linux\libxkbcommon.so.0.0.0"
if (-not (Test-Path $library)) {
    throw "Missing vendor\linux\libxkbcommon.so.0.0.0"
}
$linkDir = Join-Path $env:TEMP "runecord-linux-deps"
New-Item -ItemType Directory -Force -Path $linkDir | Out-Null
Copy-Item $library $linkDir -Force
$env:CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C link-arg=$((Join-Path $linkDir 'libxkbcommon.so.0.0.0').Replace('\', '/'))"

Push-Location $root
try {
    if (-not $SkipBuild) {
        cargo zigbuild --release -p runecord --features embedded-server --target x86_64-unknown-linux-gnu
        if ($LASTEXITCODE -ne 0) { throw "cargo zigbuild failed" }
    }
}
finally {
    Pop-Location
}

$binary = Join-Path $root "target\x86_64-unknown-linux-gnu\release\runecord"
if (-not (Test-Path $binary)) { throw "Missing $binary" }

$stage = Join-Path $root "target\linux-package"
Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
$app = Join-Path $stage "Runecord"
New-Item -ItemType Directory -Force -Path (Join-Path $app "assets") | Out-Null
Copy-Item $binary (Join-Path $app "Runecord")
Copy-Item (Join-Path $root "assets\icon.png") (Join-Path $app "assets\icon.png")

$source = Join-Path $root "dist\src"
if (-not (Test-Path (Join-Path $source "Runecord\Cargo.toml"))) {
    & (Join-Path $PSScriptRoot "stage-source.ps1") -Destination $source
}
New-Item -ItemType Directory -Force -Path (Join-Path $app "src") | Out-Null
Copy-Item (Join-Path $source "*") (Join-Path $app "src") -Recurse -Force

Copy-Item (Join-Path $root "packaging\linux\install.sh") (Join-Path $app "install.sh")
Copy-Item (Join-Path $root "packaging\linux\uninstall.sh") (Join-Path $app "uninstall.sh")
Copy-Item (Join-Path $root "packaging\linux\runecord.desktop") (Join-Path $app "runecord.desktop")
Copy-Item (Join-Path $root "LICENSE") (Join-Path $app "LICENSE")

@"
Runecord for Linux
==================

Install for the current user (no root):

    ./install.sh

That copies the binary to ~/.local/bin/runecord and adds a desktop entry.
PREFIX=/usr/local ./install.sh installs elsewhere. Remove it later with
./uninstall.sh (add --purge to delete your data too).

Or run it in place:

    ./Runecord

Requires an x86_64 Linux with glibc 2.27 or later, X11 or Wayland, and a
Vulkan-capable GPU driver. The client hosts its own backend, so the window is
the whole app.

Data lives in ~/.local/share/runecord (honouring XDG_DATA_HOME):
  runecord.db.enc   database (encrypted)
  key.bin           encryption key; back this up with the database
  server.log        server log
  backups/          daily encrypted snapshots (newest 7)
  aiedit/src/       UI source used by the AI editor

The AI editor also needs Rust 1.99.1. If cargo is not on PATH, the editor
button stays disabled. The source is included; the compiler is not.

If the window does not appear, run ./Runecord from a terminal to see the error.

The same build is available for Windows at the download page.
"@ | Set-Content -Path (Join-Path $app "README.txt") -Encoding UTF8

New-Item -ItemType Directory -Force -Path (Join-Path $root "dist") | Out-Null
$out = Join-Path $root "dist\Runecord-linux.tar.gz"
Remove-Item $out -Force -ErrorAction SilentlyContinue
& python (Join-Path $PSScriptRoot "pack-linux.py") $stage $out
if ($LASTEXITCODE -ne 0) { throw "packaging failed" }

exit 0
