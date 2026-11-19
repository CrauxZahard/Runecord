# Manage the local-only UI customization.
#
# The customized files are stored under local-ui/ (gitignored) and copied into
# the source tree. While they are applied, skip-worktree keeps them out of
# git status, diffs and commits so they cannot be pushed by accident.
param(
    [Parameter(Position = 0)]
    [ValidateSet("status", "save", "apply", "on", "off")]
    [string]$Command = "status"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$store = Join-Path $root "local-ui"

$files = @(
    "crates/client/src/ui/common.rs",
    "crates/client/src/ui/composer.rs",
    "crates/client/src/ui/forms.rs",
    "crates/client/src/ui/mod.rs",
    "crates/client/src/ui/rail.rs",
    "crates/client/src/ui/sidebar.rs",
    "crates/ui/src/theme.rs"
)

function Set-SkipWorktree {
    param([string]$Value)
    & git -C $root update-index "$Value" -- @files
    if ($LASTEXITCODE -ne 0) {
        throw "git update-index $Value failed"
    }
}

switch ($Command) {
    "save" {
        foreach ($file in $files) {
            $source = Join-Path $root $file
            if (-not (Test-Path -LiteralPath $source)) {
                throw "Missing $source"
            }
            $target = Join-Path $store $file
            New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
            Copy-Item -LiteralPath $source -Destination $target -Force
        }
        Write-Host "Saved $($files.Count) files to local-ui/."
    }
    "apply" {
        foreach ($file in $files) {
            $source = Join-Path $store $file
            if (-not (Test-Path -LiteralPath $source)) {
                throw "Missing $source. Run 'save' first."
            }
            Copy-Item -LiteralPath $source -Destination (Join-Path $root $file) -Force
        }
        Set-SkipWorktree "--skip-worktree"
        Write-Host "Applied local-ui/ and hid the files from git."
    }
    "on" {
        Set-SkipWorktree "--skip-worktree"
        Write-Host "Local UI files are hidden from git."
    }
    "off" {
        Set-SkipWorktree "--no-skip-worktree"
        Write-Host "Local UI files are visible to git again."
    }
    "status" {
        $saved = @($files | Where-Object { Test-Path -LiteralPath (Join-Path $store $_) })
        Write-Host "Store: $store ($($saved.Count)/$($files.Count) files)"
        & git -C $root ls-files -v -- @files
        & git -C $root status --short -- @files
    }
}
