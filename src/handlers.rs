use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::debug;

use crate::{utils::read_file, AppState};

#[derive(Deserialize)]
pub struct InputPrompt {
    user_query: String,
    doc_name: String,
}

pub async fn doc_names(State(state): State<AppState>) -> impl IntoResponse {
    match state.processor.vec_store.list_collections().await {
        Ok(collection_names) => (collection_names.join(",")).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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
        "one or more uploads failed".to_string().into_response()
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
