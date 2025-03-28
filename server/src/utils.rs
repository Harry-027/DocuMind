use std::{fs, io::Write};

use axum::{
    extract::{Multipart, Request},
    middleware,
    response::Response,
};
use reqwest::Client;
use serde_json::json;

use config::{Config, FileFormat};
use tracing::{debug, info};

use anyhow::{anyhow, Context, Ok, Result};

pub enum ModelKind {
    Generate,
    Embedding,
}

// log_request is the basic logging middleware for the server
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
    pub fn get_model_details(&self, kind: ModelKind) -> Result<(&String, &String)> {
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
    let yaml_data = include_str!("../env.yaml");
    let settings = Config::builder()
        .add_source(config::File::from_str(yaml_data, FileFormat::Yaml))
        .build()
        .expect("setting file is expected to read the input variables");
    let config: ConfigVar = settings.try_deserialize().unwrap();
    config
}

// extract the file content
pub fn extract_file_content(file_name: &str) -> Result<String> {
    let file_path = format!("./uploads/{}", file_name);
    match pdf_extract::extract_text(file_path) {
        std::result::Result::Ok(content) => Ok(content),
        Err(e) => {
            debug!("failed to read the file: {}", e.to_string());
            Err(anyhow::Error::new(e).context("Failed to read the file"))
        }
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
pub async fn send_request(url: &str, model_name: &str, prompt: &str) -> Result<String> {
    let client = Client::new();

    let req_body = json!({
        "model": model_name,
        "prompt": prompt,
        "stream": false,
    });

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;
    let text_result = response.text().await?;
    Ok(text_result)
}

// get the content embeddings from the embedding gen model
pub async fn get_content_embeddings(settings: ConfigVar, content: &str) -> Result<Vec<f32>> {
    let (model_url, model_name) = settings.get_model_details(ModelKind::Embedding).unwrap();
    let response = send_request(model_url.as_str(), model_name.as_str(), content).await?;

    let response_json: serde_json::Value = serde_json::from_str(response.as_str())?;

    let embeddings: Vec<f32> = response_json["embedding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    Ok(embeddings)
}

// Read the pdf file
pub async fn read_file(mut multipart: Multipart) -> Result<Vec<String>> {
    let mut uploaded_files = vec![];
    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("error status:: {} and text {}", e.status(), e.body_text()),
        )
    })? {
        if let Some(file_name) = field.file_name().map(|name| name.to_string()) {
            if !is_pdf(file_name.as_str()) {
                return Err(anyhow!("Only Pdf files are allowed"));
            }

            let mut data = Vec::new();

            while let Some(chunk) = field.chunk().await? {
                data.extend_from_slice(&chunk);
            }

            // Save to server
            let file_path = format!("./uploads/{}", file_name);
            save_file(&file_path, &data).context("error occurred while saving the file")?;
            uploaded_files.push(file_name);
        }
    }

    if uploaded_files.is_empty() {
        return Err(anyhow!("No valid PDF files were uploaded."));
    }
    Ok(uploaded_files)
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
fn save_file(file_path: &str, data: &[u8]) -> Result<()> {
    fs::create_dir_all("./uploads")?;
    let mut file = fs::File::create(file_path)?;
    file.write_all(data).context("write operation failed")?;
    Ok(())
}
