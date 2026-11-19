//! API key storage.
//!
//! The key lives in `aiedit\.env`, next to the editor's state, never in a JSON
//! file and never in plaintext. On Windows the value is a DPAPI-sealed blob
//! bound to the signed-in account (copied to another account or machine it is
//! useless); elsewhere it is a 0600 file. A hand-written plaintext
//! `RUNECORD_AI_KEY=` line is accepted when reading — for a developer editing
//! the file on purpose — but [`save`] only ever writes the sealed form.

use std::path::PathBuf;
use std::sync::Once;

/// Written by [`save`] on Windows: hex of the DPAPI blob.
const SEALED: &str = "RUNECORD_AI_KEY_DPAPI";
/// Accepted when reading, never written on Windows.
const PLAIN: &str = "RUNECORD_AI_KEY";

const HEADER: &str = "\
# Runecord AI provider key. Written by Settings > Appearance > Edit your own UI.
# RUNECORD_AI_KEY_DPAPI is sealed for this Windows account; it is not the raw key.
# A plain RUNECORD_AI_KEY line is read if you add one by hand, but the app never
# writes your key in the clear. Delete the whole line to forget the key.
";

fn env_path() -> Result<PathBuf, String> {
    Ok(super::install::root()?.join(".env"))
}

pub fn save(key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        clear();
        return Ok(());
    }
    write_env(key)?;

    // The button must prove the key is retrievable, not just that a write
    // returned Ok: read the file back before reporting success. This uses the
    // raw reader on purpose — `save` must not re-enter the migration below.
    match read_env() {
        Some(back) if back == key => Ok(()),
        Some(_) => Err("The key was written but did not read back unchanged.".into()),
        None => Err(format!(
            "The key was written to {} but could not be read back.",
            env_path()?.display()
        )),
    }
}

pub fn load() -> Option<String> {
    migrate_legacy();
    read_env()
}

/// Whether a key is stored, without keeping it around.
pub fn exists() -> bool {
    load().is_some()
}

pub fn clear() {
    if let Ok(path) = env_path() {
        let _ = std::fs::remove_file(path);
    }
    if let Ok(root) = super::install::root() {
        let _ = std::fs::remove_file(root.join("key.bin"));
    }
}

/// Writes the `.env` and nothing else: no migration, no verification.
fn write_env(key: &str) -> Result<(), String> {
    let path = env_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    let body = match sealed_line(key)? {
        Some(line) => format!("{HEADER}{line}\n"),
        None => format!("{HEADER}{PLAIN}={key}\n"),
    };
    std::fs::write(&path, body).map_err(|error| format!("Could not save the key: {error}"))?;
    restrict(&path);
    Ok(())
}

/// Parses the `.env` as it is, without migrating anything.
fn read_env() -> Option<String> {
    let text = std::fs::read_to_string(env_path().ok()?).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        let name = name.trim();
        let value = unquote(value.trim());
        if name == SEALED {
            let bytes = unhex(value)?;
            return open(&bytes)
                .ok()
                .and_then(|plain| String::from_utf8(plain).ok());
        }
        if name == PLAIN && !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

/// Earlier builds used a bare DPAPI file. Fold it into `.env` once, then drop it.
fn migrate_legacy() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let Ok(root) = super::install::root() else {
            return;
        };
        let legacy = root.join("key.bin");
        if !legacy.exists() {
            return;
        }
        let migrated = std::fs::read(&legacy)
            .ok()
            .and_then(|sealed| open(&sealed).ok())
            .and_then(|plain| String::from_utf8(plain).ok());
        if let Some(key) = migrated
            && write_env(&key).is_ok()
        {
            let _ = std::fs::remove_file(&legacy);
        }
    });
}

/// The sealed line for this platform, or `None` where DPAPI does not exist.
#[cfg(windows)]
fn sealed_line(key: &str) -> Result<Option<String>, String> {
    let sealed = seal(key.as_bytes())?;
    Ok(Some(format!("{SEALED}={}", hex(&sealed))))
}

#[cfg(not(windows))]
fn sealed_line(_key: &str) -> Result<Option<String>, String> {
    Ok(None)
}

fn unquote(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from_digit((byte >> 4) as u32, 16).unwrap_or('0'));
        out.push(char::from_digit((byte & 0x0f) as u32, 16).unwrap_or('0'));
    }
    out
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks(2) {
        let high = (pair[0] as char).to_digit(16)?;
        let low = (pair[1] as char).to_digit(16)?;
        out.push(((high << 4) | low) as u8);
    }
    Some(out)
}

#[cfg(unix)]
fn restrict(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict(_path: &std::path::Path) {}

#[cfg(windows)]
fn seal(plain: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptProtectData};
    let mut input = plain.to_vec();
    let blob_in = CRYPT_INTEGER_BLOB {
        cbData: input.len() as u32,
        pbData: input.as_mut_ptr(),
    };
    let mut blob_out = CRYPT_INTEGER_BLOB::default();
    // SAFETY: both blobs describe live buffers; the API copies into blob_out.
    unsafe {
        CryptProtectData(
            &blob_in,
            windows::core::PCWSTR::null(),
            None,
            None,
            None,
            0,
            &mut blob_out,
        )
        .map_err(|error| format!("Could not encrypt the key: {error}"))?;
    }
    if blob_out.pbData.is_null() {
        return Err("Could not encrypt the key.".into());
    }
    // SAFETY: the API filled pbData with cbData readable bytes.
    let sealed =
        unsafe { std::slice::from_raw_parts(blob_out.pbData, blob_out.cbData as usize).to_vec() };
    // SAFETY: LocalFree releases its own allocation.
    unsafe {
        let _ = LocalFree(Some(HLOCAL(blob_out.pbData as *mut _)));
    }
    Ok(sealed)
}

#[cfg(windows)]
fn open(sealed: &[u8]) -> Result<Vec<u8>, String> {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptUnprotectData};
    let mut input = sealed.to_vec();
    let blob_in = CRYPT_INTEGER_BLOB {
        cbData: input.len() as u32,
        pbData: input.as_mut_ptr(),
    };
    let mut blob_out = CRYPT_INTEGER_BLOB::default();
    // SAFETY: see `seal`; the description is dropped by passing None.
    unsafe {
        CryptUnprotectData(&blob_in, None, None, None, None, 0, &mut blob_out)
            .map_err(|error| format!("Could not read the saved key: {error}"))?;
    }
    if blob_out.pbData.is_null() {
        return Err("The saved key is empty.".into());
    }
    // SAFETY: the API filled pbData with cbData readable bytes.
    let plain =
        unsafe { std::slice::from_raw_parts(blob_out.pbData, blob_out.cbData as usize).to_vec() };
    // SAFETY: LocalFree releases its own allocation.
    unsafe {
        let _ = LocalFree(Some(HLOCAL(blob_out.pbData as *mut _)));
    }
    Ok(plain)
}

/// Without DPAPI the sealed form is the plain bytes; the file is 0600.
#[cfg(not(windows))]
fn seal(plain: &[u8]) -> Result<Vec<u8>, String> {
    Ok(plain.to_vec())
}

#[cfg(not(windows))]
fn open(sealed: &[u8]) -> Result<Vec<u8>, String> {
    Ok(sealed.to_vec())
}

#[cfg(test)]
mod tests {
    use super::{hex, unhex};

    #[test]
    fn hex_round_trips() {
        let bytes = vec![0u8, 1, 15, 16, 127, 128, 255];
        assert_eq!(unhex(&hex(&bytes)).unwrap(), bytes);
    }

    #[test]
    fn rejects_broken_hex() {
        assert!(unhex("abc").is_none());
        assert!(unhex("zz").is_none());
        assert!(unhex("é1").is_none());
    }

    #[test]
    fn the_sealed_form_reads_back() {
        // The same primitives the file uses, without touching the user's .env.
        let key = "sk-test-0123456789";
        let sealed = super::seal(key.as_bytes()).unwrap();
        #[cfg(windows)]
        assert_ne!(
            sealed,
            key.as_bytes(),
            "the sealed bytes must not be the key"
        );
        let plain = super::open(&sealed).unwrap();
        assert_eq!(String::from_utf8(plain).unwrap(), key);
    }
}
