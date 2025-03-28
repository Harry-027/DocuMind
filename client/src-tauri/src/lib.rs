use base64::decode;
use regex::Regex;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use reqwest;
use serde_json::json;

// Structs for API responses and requests
#[derive(Clone, Debug, Deserialize, Serialize)]
struct ListItem {
    name: String,
}

fn get_backend_url() -> String {
    let backend_url = std::env::var("BACKEND_URL")
       .expect("BACKEND_URL not found");
    backend_url
}

// HTTP API call to fetch list items
#[tauri::command]
async fn fetch_list_items() -> Result<Vec<ListItem>, String> {
    let backend_url = get_backend_url();
   
    match reqwest::Client::new()
        .get(backend_url)
        .send()
        .await
    {
        Ok(response) => {
                response.json::<Vec<ListItem>>()
                .await
                .map_err(|e| e.to_string())
            },
        Err(e) => Err(e.to_string())
    }
}

fn style_text(text: &str) -> Vec<String> {
    let mut styled_parts = Vec::new();
    let paragraphs = text.split("\n\n");

    let important_words = Regex::new(r"\b(important|error|warning|success|note)\b").unwrap();

    for para in paragraphs {
        if para.len() > 100 {
            let highlighted_text = important_words.replace_all(para, "<span class='highlight'>$0</span>");
            styled_parts.push(format!("<p>{}</p>", highlighted_text));
        }
    }

    styled_parts
}


#[tauri::command]
async fn fetch_content(item: ListItem) -> Result<String, String> {
    let backend_url = get_backend_url();
    match reqwest::Client::new()
       .get(&format!("{}/file/{}.pdf", backend_url, item.name))
       .send()
       .await
     {
        Ok(response) => {
           let content = response.text().await.unwrap();
           let result_content = style_text(content.as_str());
            Ok(result_content.join("<br/>"))
        },
        Err(e) => Err(e.to_string())
     }
}

#[tauri::command]
async fn process_prompt(item: ListItem, query: String) -> Result<String, String> {
    let backend_url  = get_backend_url();
    let payload = json!({
        "user_query": query,
        "doc_name": format!("{}.pdf",item.name)
    });

    match reqwest::Client::new()
       .post(&format!("{}/prompt", backend_url))
       .json(&payload)
       .send().await
      {
        Ok(response) => {
            let llm_response = response.text().await.unwrap_or_else(|e| {
               format!("unable to get the llm response:: {:?}", e.to_string())
            });
            Ok(llm_response)
        }
        Err(e) => Err(e.to_string())
      }
}

#[tauri::command]
async fn upload_file(name: String, ct: String) -> Result<String, String> {
    let backend_url  = get_backend_url();
    let decoded_data = decode(&ct).map_err(|e| format!("Base64 Decode Error: {}", e))?;

    let file_part = Part::bytes(decoded_data)
        .file_name(name.clone())
        .mime_str("application/octet-stream")
        .map_err(|e| format!("File Part Error: {}", e))?;

    let form = Form::new()
        .part("file", file_part);

        let client = reqwest::Client::new();
        match client.post(&format!("{}/upload", backend_url))
            .multipart(form)
            .send()
            .await {
                Ok(response) => {
                    if response.status().is_success() {
                        Ok("File uploaded successfully!".to_string())
                    } else {
                        Err(format!("Upload failed with status: {}", response.status()))
                    }
                },
                Err(e) => Err(format!("Request failed: {}", e)),
            }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![fetch_list_items, fetch_content, process_prompt, upload_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
