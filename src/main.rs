mod handlers;
mod processor;
mod utils;
mod vector_db;

use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use handlers::{doc_names, file_handler, prompt_handler, upload_file};
use processor::Processor;
use tracing::info;
use tracing_subscriber;
use utils::{get_settings, log_request, ConfigVar};
use vector_db::VectorStore;

#[derive(Clone)]
struct AppState {
    processor: Arc<Processor>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
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
        .route("/file/{fileName}", get(file_handler))
        .route("/upload", post(upload_file))
        .route("/prompt", post(prompt_handler))
        .layer(middleware::from_fn(log_request))
        .with_state(state.clone());

    // start the app server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Starting server at port: 3000");
    axum::serve(listener, app).await.unwrap();
}
