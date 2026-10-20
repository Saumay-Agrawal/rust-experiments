use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Clone, Debug, Deserialize)]
pub struct Model {
    pub name: String,
}

#[derive(Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<Model>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    stream: bool,
}

#[derive(Deserialize)]
struct ChatChunk {
    message: Option<ChunkMessage>,
    done: Option<bool>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ChunkMessage {
    content: Option<String>,
}

pub enum GenerationEvent {
    Token(String),
    Finished(Result<(), String>),
}

pub async fn list_models(client: &Client, base_url: &str) -> Result<Vec<Model>, String> {
    let response = client
        .get(format!("{base_url}/api/tags"))
        .send()
        .await
        .map_err(|error| format!("Could not connect to Ollama at {base_url}: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Ollama returned an error while listing models: {error}"))?;
    let mut models = response
        .json::<TagsResponse>()
        .await
        .map_err(|error| format!("Could not read Ollama's model list: {error}"))?
        .models;
    models.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(models)
}

pub async fn stream_chat(
    client: Client,
    base_url: String,
    model: String,
    messages: Vec<Message>,
    events: mpsc::UnboundedSender<GenerationEvent>,
) {
    let result = async {
        let response = client
            .post(format!("{base_url}/api/chat"))
            .json(&ChatRequest {
                model: &model,
                messages: &messages,
                stream: true,
            })
            .send()
            .await
            .map_err(|error| format!("Could not send chat request: {error}"))?
            .error_for_status()
            .map_err(|error| format!("Ollama rejected the chat request: {error}"))?;

        let mut body = response.bytes_stream();
        let mut pending = Vec::new();
        while let Some(chunk) = body.next().await {
            let chunk = chunk.map_err(|error| format!("Error reading Ollama response: {error}"))?;
            pending.extend_from_slice(&chunk);
            while let Some(newline) = pending.iter().position(|byte| *byte == b'\n') {
                let line: Vec<_> = pending.drain(..=newline).collect();
                let line = line
                    .iter()
                    .copied()
                    .filter(|byte| *byte != b'\r' && *byte != b'\n')
                    .collect::<Vec<_>>();
                if line.is_empty() {
                    continue;
                }
                let item: ChatChunk = serde_json::from_slice(&line)
                    .map_err(|error| format!("Invalid Ollama stream response: {error}"))?;
                if let Some(error) = item.error {
                    return Err(error);
                }
                if let Some(content) = item.message.and_then(|message| message.content) {
                    let _ = events.send(GenerationEvent::Token(content));
                }
                if item.done.unwrap_or(false) {
                    return Ok(());
                }
            }
        }
        if !pending.is_empty() {
            let item: ChatChunk = serde_json::from_slice(&pending)
                .map_err(|error| format!("Invalid Ollama stream response: {error}"))?;
            if let Some(error) = item.error {
                return Err(error);
            }
            if let Some(content) = item.message.and_then(|message| message.content) {
                let _ = events.send(GenerationEvent::Token(content));
            }
            if !item.done.unwrap_or(false) {
                return Err("Ollama ended the response before marking it complete".to_owned());
            }
        }
        Ok(())
    }
    .await;
    let _ = events.send(GenerationEvent::Finished(result));
}
