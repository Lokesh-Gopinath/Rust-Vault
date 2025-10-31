use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use mongodb::{bson::doc, options::ClientOptions, Client, Collection, Database};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::services::ServeDir;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    db: Database,
}

#[derive(Serialize, Deserialize)]
struct Note {
    title: String,
    content: String,
}

#[tokio::main]
async fn main() {
    // MongoDB connection
    let client_uri = std::env::var("MONGODB_URI").expect("MONGODB_URI not set");
    let client_options = ClientOptions::parse(&client_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("rustvault");

    let state = Arc::new(AppState { db });

    // CORS
    let cors = CorsLayer::new().allow_origin(Any);

    // Router
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/collections", get(get_collections))
        .route("/add/:collection", post(add_note))
        .route("/documents/:collection", get(get_notes))
        .nest_service("/static", ServeDir::new("src/web")) // serve CSS + JS
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("✅ RustVault running on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

// Serve HTML
async fn serve_index() -> impl IntoResponse {
    Html(include_str!("web/index.html"))
}

// Add new note
async fn add_note(
    Path(collection): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(note): Json<Note>,
) -> impl IntoResponse {
    let collection: Collection<Note> = state.db.collection(&collection);
    let bson_doc = doc! { "title": note.title, "content": note.content };
    collection.insert_one(bson_doc).await.unwrap();
    (StatusCode::OK, "Note added successfully")
}

// Fetch notes
async fn get_notes(
    Path(collection): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let collection: Collection<Note> = state.db.collection(&collection);
    let mut cursor = collection.find(doc! {}).await.unwrap();

    let mut notes = Vec::new();
    while let Some(Ok(note)) = cursor.next().await {
        notes.push(note);
    }

    Json(notes)
}

// Get collection names dynamically
async fn get_collections(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let names = state.db.list_collection_names().await.unwrap_or_default();
    Json(names)
}
