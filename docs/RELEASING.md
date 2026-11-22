# Releasing

The release is two artifacts from one client binary: a Windows installer and a
Linux tarball. Both are built from this checkout, with `..\rune` and
`..\Runecord-backend` present as siblings.

## Before building

```bat
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace
cargo build --workspace
```

Run the same commands in `..\Runecord-backend` when the protocol changed.

## Windows

```bat
build.bat --desktop-only
```

`build.bat` calls `scripts/build-installer.ps1`, which signs the app and the
installer. Without `-Pfx`, it creates and reuses a **self-signed** certificate
under `%LOCALAPPDATA%\Runecord\signing`. That is fine for local testing, but
SmartScreen will warn on first launch.

For a release, pass a CA-issued certificate and require a trusted signature:

```bat
powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1 ^
  -Pfx C:\path\runecord.pfx -PfxPassword "%PFX_PASSWORD%" -RequireTrusted
```

Or from `build.bat`:

```bat
build.bat --desktop-only --sign-required
```

The gate stops the build unless `Get-AuthenticodeSignature` reports `Valid`.
A self-signed certificate cannot pass it. No certificate is stored in this
repository.

## Linux

Run the smoke checks on an x86_64 Linux host with the same `..\rune` and
`..\Runecord-backend` checkouts that built the client:

```bash
bash scripts/verify-linux.sh
```

That checks formatting, runs the client tests and builds the release binary
with the embedded server. The script is local-only on purpose: the client
builds against sibling working trees, so an automatic CI job must first pin
pushed, reproducible Rune and backend refs. Do not add a hosted job that checks
out different refs and call it the release check.

The shipped `dist\Runecord-linux.tar.gz` is cross-compiled on Windows with
`scripts/build-linux.ps1` (zig + cargo-zigbuild). Cross-compiling is not a
substitute for the Linux run: do both before publishing.

## The AI editor

The editor is a preview. Check these on a clean profile:

1. Install with the installer, then confirm `<install>\src\Runecord\Cargo.toml`
   exists next to `rune` and `Runecord-backend`.
2. With Cargo not on PATH, the Appearance card disables Generate, says
   **Rust not found**, and offers **Get Rust** and **Check again**.
3. With Cargo on PATH, Generate builds a separate copy and **Restart** switches
   to it.
4. Change `app_version` in a staged `meta.json` to an older version, restart,
   and confirm the **built for an older app** warning appears with Keep,
   Rebuild and Reset. Keep silences it for this app version only.

The source bundle ships with the installer. The one-time Rust toolchain download
is still not implemented, so people without Rust must install it first.

## Checklist

- [ ] Both workspaces are formatted, checked and tested.
- [ ] The Windows app and installer report a `Valid` signature.
- [ ] The installer launches, and the portable exe launches.
- [ ] The Linux script ran on a Linux host with matching sibling checkouts.
- [ ] The installer contains `src\Runecord\Cargo.toml`, `src\rune` and `src\Runecord-backend`.
- [ ] With Cargo removed from PATH, Generate is disabled and says Rust not found.
- [ ] A clean profile shows the expected AI editor availability state.
- [ ] `dist\README.txt`, `LICENSE` and both icons are present.
- [ ] The version in `crates/client/Cargo.toml` matches the download page.
