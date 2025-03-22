use std::{
    fs,
    io::{Error, Write},
};

use axum::{
    extract::{Multipart, Request},
    middleware,
    response::Response,
};
use reqwest::Client;
use serde_json::json;

use config::Config;
use tracing::info;

pub const ENV_FILE: &str = "env.yaml";

pub enum ModelKind {
    Generate,
    Embedding,
}

pub async fn log_request(req: Request, next: middleware::Next) -> Response {
    info!("Incoming request: {} {}", req.method(), req.uri());
    let response = next.run(req).await;
    info!("Response status: {}", response.status());
    response
}

#[derive(serde_derive::Deserialize, Clone, Debug)]
pub struct ConfigVar {
    pub embedding_model_url: Option<String>,
    pub embedding_model_name: Option<String>,
    pub generate_model_url: Option<String>,
    pub generate_model_name: Option<String>,
    pub db_url: Option<String>,
    pub embedding_model_chunk_size: Option<usize>,
}

impl ConfigVar {
    //get the model details from the config var based on kind enum
    pub fn get_model_details(&self, kind: ModelKind) -> Result<(&String, &String), ()> {
        match kind {
            ModelKind::Generate => {
                let model_url = &self
                    .generate_model_url
                    .as_ref()
                    .expect("model url is expected");
                let model_name = &self
                    .generate_model_name
                    .as_ref()
                    .expect("model name is expected");
                Ok((model_url, model_name))
            }
            ModelKind::Embedding => {
                let model_url = &self
                    .embedding_model_url
                    .as_ref()
                    .expect("model url is expected");
                let model_name = &self
                    .embedding_model_name
                    .as_ref()
                    .expect("model name is expected");
                Ok((model_url, model_name))
            }
        }
    }
}

// fetch the config variables
pub fn get_settings() -> ConfigVar {
    let settings = Config::builder()
        .add_source(config::File::with_name(ENV_FILE))
        .build()
        .expect("setting file is expected to read the input variables");
    let config: ConfigVar = settings.try_deserialize().unwrap();
    config
}

// extract the file content
pub fn extract_file_content(file_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = format!("./uploads/{}", file_name);
    match pdf_extract::extract_text(file_path) {
        Ok(content) => Ok(content),
        Err(err) => Err(Box::new(Error::new(
            std::io::ErrorKind::InvalidData,
            err.to_string(),
        ))),
    }
}

// chunk the large text based on chunk size
pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    text.chars()
        .collect::<Vec<char>>()
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

// send_request helps to send the request to ollama api
pub async fn send_request(
    url: &str,
    model_name: &str,
    prompt: &str,
) -> Result<String, reqwest::Error> {
    let client = Client::new();

    let req_body = json!({
        "model": model_name,
        "prompt": prompt,
        "stream": false,
    });

    match client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await
    {
        Ok(response) => response.text().await,
        Err(err) => Err(err),
    }
}

// get the content embeddings from the embedding gen model
pub async fn get_content_embeddings(
    settings: ConfigVar,
    content: &str,
) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
    let (model_url, model_name) = settings.get_model_details(ModelKind::Embedding).unwrap();
    let response = send_request(model_url.as_str(), model_name.as_str(), content).await?;

    let response_json: serde_json::Value =
        serde_json::from_str(response.as_str()).expect("expected parsing");

    let embeddings: Vec<f32> = response_json["embedding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    Ok(embeddings)
}

// Read the pdf file
pub async fn read_file(mut multipart: Multipart) -> Result<String, Box<dyn std::error::Error>> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(file_name) = field.file_name().map(|name| name.to_string()) {
            if !is_pdf(file_name.as_str()) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Only Pdf files are allowed",
                )));
            }

            let bytes = field.bytes().await?;
            // Save to server
            let file_path = format!("./uploads/{}", file_name);
            match save_file(&file_path, &bytes) {
                Ok(_) => {
                    return Ok(file_name);
                }
                Err(_) => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Error occurred while saving the file",
                    )));
                }
            };
        }
    }
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "file doesn't exist",
    )))
}

// Validate PDF File Extension
fn is_pdf(file_name: &str) -> bool {
    std::path::Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("pdf"))
        .unwrap_or(false)
}

// Save File to Server
fn save_file(file_path: &str, data: &[u8]) -> std::io::Result<()> {
    fs::create_dir_all("./uploads")?;
    let mut file = fs::File::create(file_path)?;
    file.write_all(data)?;
    Ok(())
}
