use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use mongodb::{bson::doc, bson::Document, Client, Collection};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use futures::StreamExt; // 👈 For .next()

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    title: String,
    content: String,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/", get(index))
        .route("/collections", get(list_collections))
        .route("/add/:collection", post(add_note))
        .route("/get/:collection", get(get_notes))
        .layer(cors);

    println!("✅ RustVault running on http://0.0.0.0:8080");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Serve HTML UI
async fn index() -> impl IntoResponse {
    Html(include_str!("web/index.html"))
}

// MongoDB client helper
async fn get_client() -> Client {
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not set");
    Client::with_uri_str(uri).await.unwrap()
}

// List collections
async fn list_collections() -> impl IntoResponse {
    let client = get_client().await;
    let db = client.database("rustvault");

    let collections = db
        .list_collection_names() // ✅ Removed (None)
        .await
        .unwrap();

    Json(collections)
}

// Add note
async fn add_note(Path(collection): Path<String>, Json(note): Json<Note>) -> impl IntoResponse {
    let client = get_client().await;
    let db = client.database("rustvault");
    let collection: Collection<Document> = db.collection(&collection); // ✅ Using Document

    let bson_doc = doc! {
        "title": note.title,
        "content": note.content
    };

    collection.insert_one(bson_doc).await.unwrap(); // ✅ No extra argument
    (StatusCode::OK, "Note added").into_response()
}

// Get notes
async fn get_notes(Path(collection): Path<String>) -> impl IntoResponse {
    let client = get_client().await;
    let db = client.database("rustvault");
    let collection: Collection<Document> = db.collection(&collection);

    let mut cursor = collection.find(doc! {}).await.unwrap();
    let mut notes: Vec<Document> = Vec::new();

    while let Some(result) = cursor.next().await {
        if let Ok(note) = result {
            notes.push(note);
        }
    }

    Json(notes)
}

