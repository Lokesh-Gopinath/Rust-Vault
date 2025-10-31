use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use mongodb::{
    bson::{doc, Document},
    Client, Database,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use futures_util::StreamExt;

#[derive(Clone)]
struct AppState {
    db: Database,
}

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    title: String,
    content: String,
}

#[tokio::main]
async fn main() {
    // Connect to MongoDB
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .expect("Failed to connect to MongoDB");

    let db = client.database("rustvault");
    let state = Arc::new(AppState { db });

    // Serve static files (for /static/*)
    let static_files = ServeDir::new("src/web");

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Setup routes
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/notes", get(get_notes).post(add_note))
        .nest_service("/static", static_files)
        .with_state(state)
        .layer(cors);

    println!("🚀 Server running at http://127.0.0.1:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn serve_index() -> impl IntoResponse {
    Html(include_str!("web/index.html"))
}

async fn get_notes(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let collection = state.db.collection::<Document>("notes");

    let mut cursor = collection.find(doc! {}).await.expect("Find failed");

    let mut results = Vec::new();
    while let Some(result) = cursor.try_next().await.unwrap_or(None) {
        results.push(result);
    }

    Json(results)
}

async fn add_note(State(state): State<Arc<AppState>>, Json(note): Json<Note>) -> impl IntoResponse {
    let collection = state.db.collection::<Document>("notes");

    let doc = doc! {
        "title": note.title,
        "content": note.content,
    };

    collection.insert_one(doc, None).await.expect("Insert failed");

    Json(serde_json::json!({"status": "ok"}))
}
