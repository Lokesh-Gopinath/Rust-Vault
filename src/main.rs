use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::StreamExt; // 👈 for cursor.next()

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    title: String,
    content: String,
}

#[derive(Clone)]
struct AppState {
    collection: Arc<Collection<Note>>,
}

#[tokio::main]
async fn main() {
    let mongo_uri =
        std::env::var("MONGO_URI").expect("MONGO_URI must be set in environment variables");
    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");
    let db = client.database("rustvault");
    let collection = db.collection::<Note>("notes");

    let state = AppState {
        collection: Arc::new(collection),
    };

    let app = Router::new()
        .route("/notes", get(get_notes))
        .route("/add", axum::routing::post(add_note))
        .with_state(state);

    println!("Server running on 0.0.0.0:8080");
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn add_note(
    State(state): State<AppState>,
    Json(note): Json<Note>,
) -> Json<&'static str> {
    state
        .collection
        .insert_one(doc! { "title": note.title, "content": note.content })
        .await
        .unwrap();
    Json("Note added")
}

async fn get_notes(State(state): State<AppState>) -> Json<Vec<Note>> {
    let mut cursor = state.collection.find(doc! {}).await.unwrap();
    let mut notes = Vec::new();

    while let Some(result) = cursor.next().await {
        if let Ok(note) = result {
            notes.push(note);
        }
    }

    Json(notes)
}
