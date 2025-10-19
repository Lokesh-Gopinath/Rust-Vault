use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use mongodb::{bson::doc, bson::Document, Client, Collection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::StreamExt;
use tokio::net::TcpListener;
use axum::serve;

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    title: String,
    content: String,
}

#[derive(Clone)]
struct AppState {
    collection: Arc<Collection<Document>>,
}

#[tokio::main]
async fn main() {
    // Get MongoDB connection string from environment variable
    let mongo_uri =
        std::env::var("MONGO_URI").expect("❌ MONGO_URI must be set in environment variables");

    // Connect to MongoDB
    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("❌ Failed to connect to MongoDB");
    let db = client.database("rustvault");
    let collection = db.collection::<Document>("notes");

    let state = AppState {
        collection: Arc::new(collection),
    };

    // Define routes
    let app = Router::new()
        .route("/notes", get(get_notes))
        .route("/add", post(add_note))
        .with_state(state);

    // Bind and serve
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("❌ Failed to bind to port 8080");

    println!("✅ RustVault running on http://0.0.0.0:8080");
    serve(listener, app).await.unwrap();
}

// Add a new note
async fn add_note(
    State(state): State<AppState>,
    Json(note): Json<Note>,
) -> Json<&'static str> {
    state
        .collection
        .insert_one(doc! { "title": note.title, "content": note.content })
        .await
        .unwrap();
    Json("✅ Note added successfully")
}

// Fetch all notes
async fn get_notes(State(state): State<AppState>) -> Json<Vec<Note>> {
    let mut cursor = state.collection.find(doc! {}).await.unwrap();
    let mut notes = Vec::new();

    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            if let (Some(title), Some(content)) = (
                doc.get_str("title").ok(),
                doc.get_str("content").ok(),
            ) {
                notes.push(Note {
                    title: title.to_string(),
                    content: content.to_string(),
                });
            }
        }
    }

    Json(notes)
}
