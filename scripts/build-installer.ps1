# Builds and signs the Windows release: app exe, then the per-user installer.
#
# Self-signed by default: the certificate is created once under
# %LOCALAPPDATA%\Runecord\signing and reused. A real certificate can be supplied
# with -Pfx and -PfxPassword; the pipeline is otherwise identical.
#
#   scripts\build-installer.ps1 [-SkipBuild] [-Pfx path] [-PfxPassword value]
param(
    [switch]$SkipBuild,
    [string]$Pfx = "",
    [string]$PfxPassword = "runecord-selfsigned"
)
$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $root "dist"
$clientManifest = Join-Path $root "crates\client\Cargo.toml"
$version = (Select-String -Path $clientManifest -Pattern '^version = "([^"]+)"' |
    Select-Object -First 1).Matches[0].Groups[1].Value
if (-not $version) { throw "could not read the client version" }

function Find-Iscc {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA "Programs\Inno Setup 6\ISCC.exe"),
        "D:\tools\inno\ISCC.exe",
        "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
        "C:\Program Files\Inno Setup 6\ISCC.exe"
    )
    if ($env:INNO_SETUP_HOME) {
        $candidates = @(Join-Path $env:INNO_SETUP_HOME "ISCC.exe") + $candidates
    }
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) { return $candidate }
    }
    throw "ISCC.exe not found. Install Inno Setup 6 or set INNO_SETUP_HOME."
}

function Get-SigningCertificate {
    param([string]$Path, [string]$Password)
    $flags = [System.Security.Cryptography.X509Certificates.X509KeyStorageFlags]::EphemeralKeySet -bor
        [System.Security.Cryptography.X509Certificates.X509KeyStorageFlags]::Exportable
    return New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($Path, $Password, $flags)
}

# 1. The release binary.
if (-not $SkipBuild) {
    & cargo build --release -p runecord --features embedded-server --manifest-path (Join-Path $root "Cargo.toml")
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
}
$built = Join-Path $root "target\release\runecord.exe"
if (-not (Test-Path $built)) { throw "missing $built" }
New-Item -ItemType Directory -Force -Path (Join-Path $dist "assets") | Out-Null
# A running copy locks the old file; stop only the copy in dist.
Get-CimInstance Win32_Process -Filter "Name='Runecord.exe'" |
    Where-Object { $_.ExecutablePath -like "$dist*" } |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
Copy-Item $built (Join-Path $dist "Runecord.exe") -Force
Copy-Item (Join-Path $root "assets\icon.ico") (Join-Path $dist "assets\icon.ico") -Force
Copy-Item (Join-Path $root "assets\icon.png") (Join-Path $dist "assets\icon.png") -Force
Copy-Item (Join-Path $root "LICENSE") (Join-Path $dist "LICENSE") -Force

# 2. Certificate: use the supplied one, or create a stable self-signed one.
if ($Pfx) {
    if (-not (Test-Path $Pfx)) { throw "missing certificate $Pfx" }
    $certificate = Get-SigningCertificate -Path $Pfx -Password $PfxPassword
} else {
    $signingDir = Join-Path $env:LOCALAPPDATA "Runecord\signing"
    $pfxPath = Join-Path $signingDir "runecord-selfsigned.pfx"
    if (-not (Test-Path $pfxPath)) {
        New-Item -ItemType Directory -Force -Path $signingDir | Out-Null
        $secure = ConvertTo-SecureString -String $PfxPassword -AsPlainText -Force
        $created = New-SelfSignedCertificate `
            -Type CodeSigningCert `
            -Subject "CN=Runecord (self-signed)" `
            -KeyUsage DigitalSignature `
            -FriendlyName "Runecord self-signed code signing" `
            -CertStoreLocation "Cert:\CurrentUser\My" `
            -NotAfter (Get-Date).AddYears(5) `
            -HashAlgorithm SHA256
        Export-PfxCertificate -Cert $created -FilePath $pfxPath -Password $secure | Out-Null
        Remove-Item ("Cert:\CurrentUser\My\" + $created.Thumbprint) -ErrorAction SilentlyContinue
        Write-Output "created self-signed certificate at $pfxPath"
    }
    $certificate = Get-SigningCertificate -Path $pfxPath -Password $PfxPassword
}

function Sign-Artifact {
    param([string]$Path)
    try {
        $signature = Set-AuthenticodeSignature -FilePath $Path -Certificate $certificate `
            -HashAlgorithm SHA256 -TimestampServer "http://timestamp.digicert.com" -ErrorAction Stop
    } catch {
        # Timestamping needs the network; a signature without one still works.
        $signature = Set-AuthenticodeSignature -FilePath $Path -Certificate $certificate `
            -HashAlgorithm SHA256 -ErrorAction Stop
    }
    $check = Get-AuthenticodeSignature -FilePath $Path
    Write-Output ("signed {0}: {1} by {2}" -f `
        (Split-Path -Leaf $Path), $check.Status, $check.SignerCertificate.Subject)
    if (-not $check.SignerCertificate) { throw "no signer certificate on $Path" }
}

# 3. Sign the app, then build and sign the installer.
Sign-Artifact (Join-Path $dist "Runecord.exe")

$iscc = Find-Iscc
$script = Join-Path $root "packaging\windows\runecord.iss"
& $iscc "/DAppVersion=$version" "/DSourceDir=$dist" "/DOutputDir=$dist" $script
if ($LASTEXITCODE -ne 0) { throw "ISCC failed" }

$installer = Join-Path $dist "Runecord-Setup-$version.exe"
if (-not (Test-Path $installer)) { throw "installer was not produced" }
Sign-Artifact $installer

Write-Output ("installer: {0} ({1:N1} MB)" -f $installer, ((Get-Item $installer).Length / 1MB))
