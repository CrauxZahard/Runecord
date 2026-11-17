//! Native Windows WASAPI output for voice playback.
//!
//! Renders decoded 48 kHz mono s16 PCM into the selected endpoint through
//! shared-mode WASAPI with automatic sample-rate/channel conversion. No
//! external helper executable is involved.

#![cfg(windows)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::{
    AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM,
    AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY, DEVICE_STATE_ACTIVE, IAudioClient, IAudioRenderClient,
    IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, WAVEFORMATEX, eConsole, eRender,
};
use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
    CoUninitialize, STGM_READ,
};

struct ComGuard;

impl ComGuard {
    fn new() -> Option<Self> {
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        hr.ok().ok().map(|_| Self)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

fn device_name(device: &IMMDevice) -> Option<String> {
    unsafe {
        let store = device.OpenPropertyStore(STGM_READ).ok()?;
        let mut value = store.GetValue(&PKEY_Device_FriendlyName).ok()?;
        let out = PropVariantToStringAlloc(&value).ok();
        let _ = PropVariantClear(&mut value);
        let out = out?;
        let name = out.to_string().ok();
        CoTaskMemFree(Some(out.0 as *const _));
        name
    }
}

fn enumerator() -> Option<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok() }
}

fn select_device(name: &str) -> Option<IMMDevice> {
    let enumerator = enumerator()?;
    if !name.trim().is_empty() {
        unsafe {
            let collection = enumerator
                .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
                .ok()?;
            let count = collection.GetCount().ok()?;
            for index in 0..count {
                let Ok(device) = collection.Item(index) else {
                    continue;
                };
                if device_name(&device).is_some_and(|value| value.trim() == name.trim()) {
                    return Some(device);
                }
            }
        }
    }
    unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok() }
}

pub fn list_output_devices() -> Vec<String> {
    thread::spawn(|| {
        let Some(_com) = ComGuard::new() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        unsafe {
            let Some(enumerator) = enumerator() else {
                return out;
            };
            let Ok(collection) = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) else {
                return out;
            };
            let count = collection.GetCount().unwrap_or(0);
            for index in 0..count {
                if let Ok(device) = collection.Item(index)
                    && let Some(name) = device_name(&device)
                    && !name.trim().is_empty()
                {
                    out.push(name);
                }
            }
        }
        out
    })
    .join()
    .unwrap_or_default()
}

pub struct WasapiPlayer {
    _com: ComGuard,
    client: IAudioClient,
    render: IAudioRenderClient,
    buffer_frames: u32,
    stop: Arc<AtomicBool>,
    pending: Vec<u8>,
}

impl WasapiPlayer {
    pub fn start(device_name: &str, stop: Arc<AtomicBool>) -> Option<Self> {
        let com = ComGuard::new()?;
        let device = select_device(device_name)?;
        let client: IAudioClient = unsafe { device.Activate(CLSCTX_ALL, None).ok()? };
        let format = WAVEFORMATEX {
            wFormatTag: 1,
            nChannels: 1,
            nSamplesPerSec: 48_000,
            wBitsPerSample: 16,
            nBlockAlign: 2,
            nAvgBytesPerSec: 96_000,
            cbSize: 0,
        };
        let flags = AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY;
        unsafe {
            client
                .Initialize(AUDCLNT_SHAREMODE_SHARED, flags, 2_000_000, 0, &format, None)
                .ok()?;
        }
        let buffer_frames = unsafe { client.GetBufferSize().ok()? };
        let render: IAudioRenderClient = unsafe { client.GetService().ok()? };
        unsafe { client.Start().ok()? };
        Some(Self {
            _com: com,
            client,
            render,
            buffer_frames,
            stop,
            pending: Vec::new(),
        })
    }

    pub fn write(&mut self, pcm: &[u8], deafened: bool) {
        self.pending.extend_from_slice(pcm);
        let frames = self.pending.len() / 2;
        let mut written = 0usize;
        while written < frames {
            if self.stop.load(Ordering::Relaxed) {
                break;
            }
            let padding = unsafe { self.client.GetCurrentPadding() }.unwrap_or(0);
            let free = self.buffer_frames.saturating_sub(padding);
            if free == 0 {
                thread::sleep(Duration::from_millis(3));
                continue;
            }
            let chunk = free
                .min((frames - written) as u32)
                .min(self.buffer_frames.max(1));
            let bytes = chunk as usize * 2;
            let Ok(target) = (unsafe { self.render.GetBuffer(chunk) }) else {
                break;
            };
            unsafe {
                if deafened {
                    std::ptr::write_bytes(target, 0, bytes);
                } else {
                    std::ptr::copy_nonoverlapping(
                        self.pending[written * 2..].as_ptr(),
                        target,
                        bytes,
                    );
                }
            }
            let _ = unsafe { self.render.ReleaseBuffer(chunk, 0) };
            written += chunk as usize;
        }
        if written > 0 {
            self.pending.drain(..written * 2);
        }
        if self.pending.len() > 48_000 {
            self.pending.drain(..self.pending.len() - 48_000);
        }
    }

    pub fn finish(self) {
        let _ = unsafe { self.client.Stop() };
    }
}

// ---------------------------------------------------------------------------
// Loopback capture (screen-share audio)
// ---------------------------------------------------------------------------

use windows::Win32::Media::Audio::{AUDCLNT_STREAMFLAGS_LOOPBACK, IAudioCaptureClient};

/// Captures whatever the machine is playing on the default render endpoint.
/// Frames come out as 48 kHz mono s16 so they can be mixed into the outgoing
/// voice stream exactly like the soundboard.
pub struct WasapiLoopback {
    _com: ComGuard,
    client: IAudioClient,
    capture: IAudioCaptureClient,
    channels: usize,
    rate: u32,
    float: bool,
    position: f64,
    previous: f32,
    stop: Arc<AtomicBool>,
}

impl WasapiLoopback {
    pub fn start(stop: Arc<AtomicBool>) -> Option<Self> {
        let com = ComGuard::new()?;
        let device = select_device("")?;
        let client: IAudioClient = unsafe { device.Activate(CLSCTX_ALL, None).ok()? };
        let raw = unsafe { client.GetMixFormat().ok()? };
        let format = unsafe { *raw };
        let channels = format.nChannels.max(1) as usize;
        let rate = format.nSamplesPerSec.max(1);
        let float = format.wFormatTag == 3;
        unsafe {
            client
                .Initialize(
                    AUDCLNT_SHAREMODE_SHARED,
                    AUDCLNT_STREAMFLAGS_LOOPBACK,
                    0,
                    0,
                    &format,
                    None,
                )
                .ok()?;
        }
        let capture: IAudioCaptureClient = unsafe { client.GetService().ok()? };
        unsafe { client.Start().ok()? };
        Some(Self {
            _com: com,
            client,
            capture,
            channels,
            rate,
            float,
            position: 0.0,
            previous: 0.0,
            stop,
        })
    }

    /// Drains the loopback buffer, returning mono 48 kHz s16 samples.
    pub fn read(&mut self) -> Vec<i16> {
        let mut out = Vec::new();
        while !self.stop.load(Ordering::Relaxed) {
            let pending = unsafe { self.capture.GetNextPacketSize() }.unwrap_or(0);
            if pending == 0 {
                break;
            }
            let mut data: *mut u8 = std::ptr::null_mut();
            let mut frames = 0u32;
            let mut flags = 0u32;
            if unsafe {
                self.capture
                    .GetBuffer(&mut data, &mut frames, &mut flags, None, None)
            }
            .is_err()
            {
                break;
            }
            if frames > 0 && !data.is_null() && flags & 0x2 == 0 {
                let mono = unsafe { self.mono(data as *const f32 as *const u8, frames as usize) };
                out.extend(self.resample(&mono));
            }
            unsafe {
                let _ = self.capture.ReleaseBuffer(frames);
            }
        }
        out
    }

    unsafe fn mono(&self, data: *const u8, frames: usize) -> Vec<f32> {
        let mut mono = Vec::with_capacity(frames);
        for frame in 0..frames {
            let mut sum = 0.0f32;
            for channel in 0..self.channels {
                let index = frame * self.channels + channel;
                let value = if self.float {
                    unsafe { *(data as *const f32).add(index) }
                } else {
                    unsafe { *(data as *const i16).add(index) as f32 / 32768.0 }
                };
                sum += value;
            }
            mono.push(sum / self.channels as f32);
        }
        mono
    }

    /// Linear resample to 48 kHz, carrying phase across buffers.
    fn resample(&mut self, input: &[f32]) -> Vec<i16> {
        if input.is_empty() {
            return Vec::new();
        }
        if self.rate == 48_000 {
            return input
                .iter()
                .map(|sample| (sample.clamp(-1.0, 1.0) * 32767.0) as i16)
                .collect();
        }
        let step = self.rate as f64 / 48_000.0;
        let mut out = Vec::new();
        let mut position = self.position;
        while position < input.len() as f64 {
            let index = position.floor() as usize;
            let fraction = (position - index as f64) as f32;
            let current = input[index.min(input.len() - 1)];
            let next = input.get(index + 1).copied().unwrap_or(current);
            let sample = self.previous + (current - self.previous) * fraction;
            out.push((sample.clamp(-1.0, 1.0) * 32767.0) as i16);
            self.previous = current;
            let _ = next;
            position += step;
        }
        self.position = position - input.len() as f64;
        out
    }
}

impl Drop for WasapiLoopback {
    fn drop(&mut self) {
        unsafe {
            let _ = self.client.Stop();
        }
    }
}
