# Stages the client, Rune and backend source trees for a packaged install.
#
# The bundled client builds against sibling folders, so the layout must stay:
#
#   <destination>\Runecord
#   <destination>\rune
#   <destination>\Runecord-backend
#
# Runtime data, build output and VCS metadata are excluded.
param(
    [Parameter(Mandatory = $true)]
    [string]$Destination
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$workspace = Split-Path -Parent $root
$rune = Join-Path $workspace "rune"
$backend = Join-Path $workspace "Runecord-backend"

foreach ($path in @($root, $rune, $backend)) {
    if (-not (Test-Path (Join-Path $path "Cargo.toml"))) {
        throw "Missing source workspace: $path"
    }
}

function Copy-Tree {
    param(
        [string]$From,
        [string]$To,
        [string[]]$Exclude
    )
    New-Item -ItemType Directory -Force -Path $To | Out-Null
    $arguments = @($From, $To, "/E", "/NFL", "/NDL", "/NJH", "/NJS", "/NP")
    if ($Exclude.Count -gt 0) {
        $arguments += "/XD"
        $arguments += $Exclude
    }
    & robocopy @arguments | Out-Null
    if ($LASTEXITCODE -ge 8) {
        throw "robocopy failed for $From (exit $LASTEXITCODE)"
    }
}

if (Test-Path $Destination) {
    Remove-Item $Destination -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $Destination | Out-Null

# The client needs its manifests, crates, assets and vendor/xkbcommon.
Copy-Tree $root (Join-Path $Destination "Runecord") @(
    (Join-Path $root ".git"), (Join-Path $root "target"), (Join-Path $root "dist"),
    (Join-Path $root "shots"), (Join-Path $root "aiedit"), (Join-Path $root "docs"),
    (Join-Path $root "packaging"), (Join-Path $root "scripts"),
    (Join-Path $root ".github"), (Join-Path $root ".runecord"),
    (Join-Path $root "promo")
)
Remove-Item (Join-Path $Destination "Runecord\AGENTS.md") -Force -ErrorAction SilentlyContinue

# Rune is a path dependency. Keep every workspace member, but drop its demo,
# research and website trees. Remove .cargo: it normally points at an absolute
# build cache.
Copy-Tree $rune (Join-Path $Destination "rune") @(
    (Join-Path $rune ".git"), (Join-Path $rune "target"),
    (Join-Path $rune "demo"), (Join-Path $rune "research"),
    (Join-Path $rune "spikes"), (Join-Path $rune "site"),
    (Join-Path $rune "changelog"), (Join-Path $rune ".cargo"),
    "node_modules"
)

# The embedded server compiles from the sibling backend. Migrations are part
# of the compile.
Copy-Tree $backend (Join-Path $Destination "Runecord-backend") @(
    (Join-Path $backend ".git"), (Join-Path $backend "target"),
    (Join-Path $backend "docs"), (Join-Path $backend "data"),
    (Join-Path $backend ".github")
)

$size = (Get-ChildItem $Destination -Recurse -File |
    Measure-Object Length -Sum).Sum
Write-Output ("source bundle: {0:N1} MB at {1}" -f ($size / 1MB), $Destination)
