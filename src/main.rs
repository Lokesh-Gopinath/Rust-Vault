use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use mongodb::{Client, Database, bson::doc};
use tower_http::cors::{Any, CorsLayer};
use dotenvy::dotenv;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    title: String,
    content: String,
}

#[derive(Clone)]
struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Read MongoDB URI from environment variable
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI must be set");
    let client = Client::with_uri_str(&mongo_uri).await.unwrap();
    let db = client.database("rustvault");

    let app_state = Arc::new(AppState { db });

    // Set up routes
    let app = Router::new()
        .route("/", get(root))
        .route("/api/notes", get(get_notes).post(add_note))
        .with_state(app_state)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any));

    let port = 8080;
    println!("🚀 Server running at http://localhost:{port}");
    axum::serve(
        tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap(),
        app,
    )
    .await
    .unwrap();
}

async fn root() -> &'static str {
    "RustVault is running!"
}

async fn add_note(
    State(state): State<Arc<AppState>>,
    Json(note): Json<Note>,
) -> Json<&'static str> {
    let collection = state.db.collection::<Note>("notes");
    let _ = collection
        .insert_one(doc! { "title": note.title, "content": note.content }, None)
        .await;
    Json("✅ Note added!")
}

async fn get_notes(State(state): State<Arc<AppState>>) -> Json<Vec<Note>> {
    let collection = state.db.collection::<Note>("notes");
    let mut cursor = collection.find(None, None).await.unwrap();
    let mut notes = Vec::new();

    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            let title = doc.get_str("title").unwrap_or("").to_string();
            let content = doc.get_str("content").unwrap_or("").to_string();
            notes.push(Note { title, content });
        }
    }

    Json(notes)
}
