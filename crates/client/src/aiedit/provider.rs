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
    let mut response = send(&client, &url, cfg, messages, streamed)?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        // Some gateways refuse `stream`; retry once without it.
        if streamed && status.as_u16() == 400 && text.to_ascii_lowercase().contains("stream") {
            super::log("The provider refused streaming; retrying without it.");
            streamed = false;
            response = send(&client, &url, cfg, messages, streamed)?;
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
    stream: bool,
) -> Result<reqwest::blocking::Response, String> {
    let body = payload(cfg, messages, stream);
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

fn payload(cfg: &Config, messages: &[(String, String)], stream: bool) -> serde_json::Value {
    serde_json::json!({
        "model": cfg.model.trim(),
        "temperature": 0.2,
        "stream": stream,
        "messages": messages
            .iter()
            .map(|(role, content)| serde_json::json!({ "role": role, "content": content }))
            .collect::<Vec<_>>(),
    })
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
    use super::{consume_event, parse_message, read_stream};

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
}
