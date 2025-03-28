use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    utils::{extract_file_content, read_file},
    AppState,
};

#[derive(Deserialize)]
pub struct InputPrompt {
    user_query: String,
    doc_name: String,
}

#[derive(Serialize)]
pub struct DocInfo {
    name: String,
}

pub async fn doc_names(State(state): State<AppState>) -> impl IntoResponse {
    match state.processor.vec_store.list_collections().await {
        Ok(collection_names) => {
            let result: Vec<DocInfo> = collection_names
                .iter()
                .map(|name| DocInfo {
                    name: name.to_string(),
                })
                .collect();
            Json(result).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn file_handler(Path(file_name): Path<String>) -> impl IntoResponse {
    match extract_file_content(file_name.as_str()) {
        Ok(data) => data.into_response(),
        Err(_) => {
            return (StatusCode::NOT_FOUND, "File not found or cannot be read.").into_response()
        }
    }
}

pub async fn upload_file(State(state): State<AppState>, multipart: Multipart) -> impl IntoResponse {
    let file_names = match read_file(multipart).await {
        Ok(file_names) => file_names,
        Err(e) => {
            debug!("unable to read the file: {}", e);
            vec![]
        }
    };
    if file_names.len() == 0 {
        return (StatusCode::BAD_REQUEST, "no pdf files were uploaded").into_response();
    }
    let mut processed_files = vec![];
    for file_name in file_names.iter() {
        match state.processor.process_file(file_name.as_str()).await {
            Ok(()) => processed_files.push(file_name),
            Err(e) => {
                eprintln!("error occurred:: {}", e.to_string());
            }
        };
    }

    if processed_files.len() == file_names.len() {
        (StatusCode::OK, "files uploaded successfully").into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "one or more uploads failed".to_string(),
        )
            .into_response()
    }
}

pub async fn prompt_handler(
    State(state): State<AppState>,
    Json(data): Json<InputPrompt>,
) -> impl IntoResponse {
    let user_query = data.user_query;
    let doc_name = data.doc_name;
    let processor = state.processor;
    match processor
        .process_prompt(user_query.as_str(), doc_name.as_str())
        .await
    {
        Ok(response) => (StatusCode::OK, response).into_response(),
        Err(e) => {
            eprintln!("error occurred:: {}", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
