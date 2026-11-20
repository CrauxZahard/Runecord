//! OpenAI-compatible chat completions.
//!
//! Streams by default. A non-streaming call has to hold a socket open for the
//! whole generation, and a thinking model can take minutes per turn; gateways
//! cut those connections, which looked like "error sending request". With
//! `stream: true` the first token arrives quickly and the body keeps flowing.
//! A gateway that rejects streaming gets one retry without it.
//!
//! Runs on a worker thread; a blocking client here never touches the UI thread.

use std::io::{BufRead, BufReader};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Config {
    pub base_url: String,
    pub model: String,
    pub key: String,
}

/// One image attached to the conversation, in the form a vision model expects.
#[derive(Clone)]
pub struct Image {
    pub media_type: String,
    pub bytes: Vec<u8>,
}

/// opencode's gateway routes by session: a request without this header is
/// rejected with `MissingSessionID` before the key is even checked. One id per
/// process keeps its cache lanes stable.
fn opencode_session() -> &'static str {
    static ID: OnceLock<String> = OnceLock::new();
    ID.get_or_init(|| {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(0);
        format!("runecord-{nanos:x}-{:x}", std::process::id())
    })
}

/// Sends the conversation and returns the assistant's text.
pub fn ask(cfg: &Config, messages: &[(String, String)]) -> Result<String, String> {
    ask_images(cfg, messages, &[])
}

/// Sends the conversation with reference images attached to the last user turn.
///
/// A provider that does not accept images gets one text-only retry: losing the
/// reference is better than losing the edit. Providers that reject the user
/// message for another reason still report that reason.
pub fn ask_images(
    cfg: &Config,
    messages: &[(String, String)],
    images: &[Image],
) -> Result<String, String> {
    if cfg.key.trim().is_empty() {
        return Err("No API key saved. Add one above and try again.".into());
    }
    if cfg.model.trim().is_empty() {
        return Err("No model set. Add one above, for example deepseek-v4.1-flash.".into());
    }
    let base = cfg.base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err("No base URL set. It should look like https://api.openai.com/v1.".into());
    }
    let url = format!("{base}/chat/completions");
    let client = reqwest::blocking::Client::builder()
        // Generous: with streaming, the body keeps arriving, so this only caps a
        // genuinely stuck call.
        .timeout(Duration::from_secs(1_800))
        .connect_timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(0)
        .user_agent(format!("Runecord/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("Could not start the request: {error}"))?;

    let mut streamed = true;
    let mut images = images;
    let mut response = send(&client, &url, cfg, messages, images, streamed)?;
    // A text-only model rejects the whole turn when it sees an image part.
    // Drop the references once and keep the idea; the brief can still describe
    // what the user asked for.
    if !response.status().is_success() && !images.is_empty() {
        let status = response.status();
        // Only content errors are worth dropping the images for. A 5xx or 429
        // is a provider problem that a second request would not fix.
        if matches!(status.as_u16(), 400 | 404 | 413 | 415 | 422) {
            super::log(&format!(
                "The provider refused the reference image ({status}); continuing without it."
            ));
            images = &[];
            response = send(&client, &url, cfg, messages, images, streamed)?;
        }
    }
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        // Some gateways refuse `stream`; retry once without it.
        if streamed && status.as_u16() == 400 && text.to_ascii_lowercase().contains("stream") {
            super::log("The provider refused streaming; retrying without it.");
            streamed = false;
            response = send(&client, &url, cfg, messages, images, streamed)?;
            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().unwrap_or_default();
                return Err(format!(
                    "The provider answered HTTP {status}: {}",
                    snippet(&text, 400)
                ));
            }
        } else {
            return Err(format!(
                "The provider answered HTTP {status}: {}",
                snippet(&text, 400)
            ));
        }
    }

    if streamed && is_event_stream(&response) {
        let text = read_stream(BufReader::new(response))?;
        if text.trim().is_empty() {
            return Err("The provider's stream ended without any content.".into());
        }
        return Ok(text);
    }

    let text = response
        .text()
        .map_err(|error| format!("Could not read the reply: {error}"))?;
    parse_message(&text)
}

/// POSTs once, retrying only failures where the provider cannot have acted:
/// a dropped connection before the response, not a timeout mid-generation.
fn send(
    client: &reqwest::blocking::Client,
    url: &str,
    cfg: &Config,
    messages: &[(String, String)],
    images: &[Image],
    stream: bool,
) -> Result<reqwest::blocking::Response, String> {
    let body = payload(cfg, messages, images, stream);
    let mut last_error = String::new();
    for attempt in 0..3 {
        let mut request = client
            .post(url)
            .header("Authorization", format!("Bearer {}", cfg.key.trim()));
        if url.contains("opencode.ai") {
            request = request.header("x-opencode-session", opencode_session());
        }
        match request.json(&body).send() {
            Ok(answer) => return Ok(answer),
            Err(error) => {
                let timeout = error.is_timeout();
                last_error = format!("Could not reach {url}: {error}");
                if timeout || attempt == 2 {
                    break;
                }
                super::log(&format!(
                    "The request failed to send (attempt {}); retrying.",
                    attempt + 1
                ));
                std::thread::sleep(Duration::from_secs(2 + attempt as u64 * 3));
            }
        }
    }
    Err(last_error)
}

fn payload(
    cfg: &Config,
    messages: &[(String, String)],
    images: &[Image],
    stream: bool,
) -> serde_json::Value {
    let last_user = messages
        .iter()
        .rposition(|(role, _)| role == "user")
        .unwrap_or(usize::MAX);
    let messages: Vec<serde_json::Value> = messages
        .iter()
        .enumerate()
        .map(|(index, (role, content))| {
            if index == last_user && !images.is_empty() {
                let mut parts = vec![serde_json::json!({ "type": "text", "text": content })];
                parts.extend(images.iter().map(|image| {
                    serde_json::json!({
                        "type": "image_url",
                        "image_url": { "url": data_url(image) },
                    })
                }));
                serde_json::json!({ "role": role, "content": parts })
            } else {
                serde_json::json!({ "role": role, "content": content })
            }
        })
        .collect();
    serde_json::json!({
        "model": cfg.model.trim(),
        "temperature": 0.2,
        "stream": stream,
        "messages": messages,
    })
}

/// The `data:` URL an OpenAI-compatible vision request expects.
fn data_url(image: &Image) -> String {
    format!(
        "data:{};base64,{}",
        image.media_type.trim(),
        base64_encode(&image.bytes)
    )
}

/// Minimal standard base64, so a reference image does not pull in a dependency
/// for four lines of arithmetic.
fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(b2 & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn is_event_stream(response: &reqwest::blocking::Response) -> bool {
    response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("event-stream"))
}

/// Reads an SSE body and accumulates the assistant's text.
fn read_stream(mut reader: impl BufRead) -> Result<String, String> {
    let mut out = String::new();
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|error| format!("The reply stream broke: {error}"))?;
        if read == 0 {
            break;
        }
        if let Some(done) = consume_event(line.trim_end(), &mut out)? {
            if done {
                break;
            }
        }
    }
    Ok(out)
}

/// Handles one SSE line. `Ok(Some(true))` means `[DONE]`.
fn consume_event(line: &str, out: &mut String) -> Result<Option<bool>, String> {
    let Some(data) = line.strip_prefix("data:") else {
        return Ok(None);
    };
    let data = data.trim();
    if data.is_empty() {
        return Ok(None);
    }
    if data == "[DONE]" {
        return Ok(Some(true));
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(data) else {
        // Keep-alive comments and partial frames are not errors.
        return Ok(None);
    };
    if let Some(reason) = value["error"]["message"].as_str() {
        return Err(format!("The provider refused the request: {reason}"));
    }
    if let Some(choices) = value["choices"].as_array() {
        for choice in choices {
            append_content(&choice["delta"]["content"], out);
            append_content(&choice["message"]["content"], out);
            append_content(&choice["text"], out);
        }
    }
    Ok(None)
}

/// Content is a string in the common case, but gateways also send parts.
fn append_content(value: &serde_json::Value, out: &mut String) {
    if let Some(text) = value.as_str() {
        out.push_str(text);
    } else if let Some(parts) = value.as_array() {
        for part in parts {
            if let Some(text) = part["text"].as_str() {
                out.push_str(text);
            } else if let Some(text) = part.as_str() {
                out.push_str(text);
            }
        }
    }
}

/// The non-streaming body: one JSON message, or an error.
fn parse_message(text: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|error| format!("The provider sent something that is not JSON: {error}"))?;
    if let Some(reason) = value["error"]["message"].as_str() {
        return Err(format!("The provider refused the request: {reason}"));
    }
    let content = &value["choices"][0]["message"]["content"];
    if let Some(text) = content.as_str() {
        return Ok(text.to_string());
    }
    if let Some(parts) = content.as_array() {
        let mut out = String::new();
        append_content(&serde_json::Value::Array(parts.clone()), &mut out);
        if !out.is_empty() {
            return Ok(out);
        }
    }
    Err(format!(
        "The reply has no message content: {}",
        snippet(text, 300)
    ))
}

/// A short, char-safe excerpt for an error message.
fn snippet(text: &str, limit: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= limit {
        return trimmed.to_string();
    }
    let cut: String = trimmed.chars().take(limit).collect();
    format!("{cut}…")
}

#[cfg(test)]
mod tests {
    use super::{Config, Image, base64_encode, consume_event, parse_message, payload, read_stream};

    #[test]
    fn accumulates_streamed_deltas() {
        let body = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n",
            "data: [DONE]\n\n",
        );
        let text = read_stream(std::io::Cursor::new(body)).expect("stream parses");
        assert_eq!(text, "Hello");
    }

    #[test]
    fn ignores_comments_and_keep_alives() {
        let body = ": ping\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n";
        let text = read_stream(std::io::Cursor::new(body)).expect("stream parses");
        assert_eq!(text, "ok");
    }

    #[test]
    fn surfaces_a_streamed_error() {
        let body = "data: {\"error\":{\"message\":\"quota exhausted\"}}\n\n";
        let error = read_stream(std::io::Cursor::new(body)).unwrap_err();
        assert!(error.contains("quota exhausted"), "{error}");
    }

    #[test]
    fn reads_parts_instead_of_a_string() {
        let body =
            "{\"choices\":[{\"message\":{\"content\":[{\"text\":\"a\"},{\"text\":\"b\"}]}}]}";
        assert_eq!(parse_message(body).unwrap(), "ab");
    }

    #[test]
    fn done_ends_the_stream() {
        let mut out = String::new();
        assert_eq!(consume_event("data: [DONE]", &mut out).unwrap(), Some(true));
    }

    #[test]
    fn base64_matches_the_standard_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn images_attach_to_the_last_user_turn_only() {
        let config = Config {
            base_url: "http://stub.invalid/v1".into(),
            model: "stub".into(),
            key: "stub".into(),
        };
        let messages = vec![
            ("system".to_string(), "rules".to_string()),
            ("user".to_string(), "make it pink".to_string()),
        ];
        let image = Image {
            media_type: "image/png".into(),
            bytes: b"ab".to_vec(),
        };
        let body = payload(&config, &messages, &[image], false);
        assert_eq!(body["messages"][0]["content"], "rules");
        let parts = body["messages"][1]["content"]
            .as_array()
            .expect("a vision message is an array of parts");
        assert_eq!(parts[0]["type"], "text");
        assert_eq!(parts[1]["type"], "image_url");
        assert_eq!(parts[1]["image_url"]["url"], "data:image/png;base64,YWI=");
    }

    #[test]
    fn a_text_only_request_keeps_string_content() {
        let config = Config {
            base_url: "http://stub.invalid/v1".into(),
            model: "stub".into(),
            key: "stub".into(),
        };
        let messages = vec![("user".to_string(), "hello".to_string())];
        let body = payload(&config, &messages, &[], false);
        assert_eq!(body["messages"][0]["content"], "hello");
    }
}
