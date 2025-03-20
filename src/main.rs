mod handlers;
mod processor;
mod utils;
mod vector_db;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use handlers::{doc_names, prompt_handler, upload_file};
use processor::Processor;
use utils::{get_settings, ConfigVar};
use vector_db::VectorStore;

#[derive(Clone)]
struct AppState {
    processor: Arc<Processor>,
}

#[tokio::main]
async fn main() {
    // fetch the env configured variables
    let settings: ConfigVar = get_settings();

    let db_url = settings
        .db_url
        .as_ref()
        .expect("db url connection string is required");

    // shared app state for handlers
    let state = AppState {
        processor: Arc::new(Processor::new(settings.clone(), VectorStore::new(&db_url))),
    };

    // the routes configuration
    let app = Router::new()
        .route("/", get(doc_names))
        .route("/upload", post(upload_file))
        .with_state(state.clone())
        .route("/prompt", post(prompt_handler))
        .with_state(state.clone());

    // start the app server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
