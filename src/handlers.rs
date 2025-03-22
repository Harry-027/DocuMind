use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{utils::read_file, AppState};

#[derive(Deserialize)]
pub struct InputPrompt {
    user_query: String,
    doc_name: String,
}

pub async fn doc_names(State(state): State<AppState>) -> impl IntoResponse {
    let collection_names = state.processor.vec_store.list_collections().await.unwrap();
    (collection_names.join(",")).into_response()
}

pub async fn upload_file(State(state): State<AppState>, multipart: Multipart) -> impl IntoResponse {
    let file_name = read_file(multipart).await.unwrap();
    match state.processor.process_file(file_name.as_str()).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            eprintln!("error occurred:: {}", e.to_string());
            e.to_string().into_response()
        }
    };
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
