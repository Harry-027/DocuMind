use std::io::Error;

use reqwest::Client;
use serde_json::json;

use config::Config;

pub const ENV_FILE: &str = "env.yaml";

#[derive(serde_derive::Deserialize, Clone, Debug)]
pub struct ConfigVar {
    pub embedding_model_url: Option<String>,
    pub embedding_model_name: Option<String>,
    pub generate_model_url: Option<String>,
    pub generate_model_name: Option<String>,
    pub db_url: Option<String>,
    pub embedding_model_chunk_size: Option<usize>,
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
    let file_path = format!("uploads/{}", file_name);
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

// get the embeddings from the model
pub async fn get_embeddings(
    settings: &ConfigVar,
    content: &str,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let embedding_url = settings
        .embedding_model_url
        .as_ref()
        .expect("embedding model string is required");
    let embedding_model_name = settings
        .embedding_model_name
        .as_ref()
        .expect("embedding model name is expected");
    let response = send_request(
        embedding_url.as_str(),
        embedding_model_name.as_str(),
        content,
    )
    .await?;

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
