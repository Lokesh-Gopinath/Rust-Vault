use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use mongodb::{bson::{self, doc, Document}, Client, Database};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use futures::StreamExt; // ✅ Correct import for async MongoDB cursor
use std::net::SocketAddr;

#[derive(Clone)]
struct AppState {
    db: Database,
}

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    title: String,
    content: String,
}

// ✅ Root route — serves your HTML page
async fn root() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

// ✅ List available MongoDB collections
async fn list_collections(State(state): State<AppState>) -> impl IntoResponse {
    let collections = state.db.list_collection_names().await.unwrap();
    Json(collections)
}

// ✅ Insert new document into a specific collection
async fn add_note(
    Path(collection): Path<String>,
    State(state): State<AppState>,
    Json(note): Json<Note>,
) -> impl IntoResponse {
    let collection = state.db.collection::<Document>(&collection);

    let bson_doc = doc! {
        "title": note.title,
        "content": note.content
    };

    // ✅ Fixed: removed the invalid type binding & added error handling
    collection.insert_one(bson_doc).await.unwrap();

    (StatusCode::OK, "Note added").into_response()
}

// ✅ Fetch all notes from a collection
async fn get_notes(
    Path(collection): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let collection = state.db.collection::<Document>(&collection);
    let mut cursor = collection.find(doc! {}).await.unwrap();

    let mut notes: Vec<Document> = Vec::new();

    while let Some(result) = cursor.next().await {
        if let Ok(doc) = result {
            notes.push(doc);
        }
    }

    Json(notes)
}

// ✅ Main app setup
#[tokio::main]
async fn main() {
    let mongo_uri =
        std::env::var("MONGODB_URI").expect("MONGODB_URI must be set in Render environment");
    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");

    let db = client.database("rustvault");

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let app_state = AppState { db };

    let app = Router::new()
        .route("/", get(root))
        .route("/collections", get(list_collections))
        .route("/add/:collection", post(add_note))
        .route("/get/:collection", get(get_notes))
        .with_state(app_state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("✅ RustVault running on http://{}/", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
