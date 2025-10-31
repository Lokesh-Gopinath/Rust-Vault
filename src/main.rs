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
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::{Any, CorsLayer}, services::ServeDir};
use futures_util::TryStreamExt; // 👈 Correct trait for Mongo cursor

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

    // Serve static files
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

    // Use new Axum 0.7 server style
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index() -> impl IntoResponse {
    Html(include_str!("web/index.html"))
}

async fn get_notes(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let collection = state.db.collection::<Document>("notes");

    let mut cursor = collection.find(doc! {}).await.expect("Find failed");
    let mut results = Vec::new();

    while let Some(result) = cursor.try_next().await.expect("Cursor failed") {
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

    collection.insert_one(doc).await.expect("Insert failed");

    Json(serde_json::json!({"status": "ok"}))
}
