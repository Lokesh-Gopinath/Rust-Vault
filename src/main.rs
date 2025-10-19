use axum::{
    extract::{Path, State},
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use futures::StreamExt;
use mongodb::{bson::{doc, Document}, Client, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use axum::serve;

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    title: String,
    content: String,
}

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

#[tokio::main]
async fn main() {
    // MongoDB URI from environment variable
    let mongo_uri =
        std::env::var("MONGO_URI").expect("MONGO_URI must be set in environment variables");

    // Connect to MongoDB
    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");
    let db = client.database("rustvault");
    let state = AppState {
        db: Arc::new(db),
    };

    // CORS layer
    let cors = CorsLayer::new().allow_origin(Any);

    // Routes
    let app = Router::new()
        .route("/", get(root))
        .route("/collections", get(get_collections))
        .route("/documents/:collection", get(get_documents))
        .route("/add/:collection", post(add_document))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(cors));

    // Bind to port
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind port 8080");

    println!("✅ RustVault running on http://0.0.0.0:8080");
    serve(listener, app).await.unwrap();
}

// Serve frontend HTML
async fn root() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

// List all collections in the database
async fn get_collections(State(state): State<AppState>) -> Json<Vec<String>> {
    let names = state
        .db
        .list_collection_names(None)
        .await
        .unwrap_or_default();
    Json(names)
}

// Add a document to selected collection
async fn add_document(
    Path(collection_name): Path<String>,
    State(state): State<AppState>,
    Json(note): Json<Note>,
) -> Json<&'static str> {
    let collection = state.db.collection::<Document>(&collection_name);
    let bson_doc = doc! { "title": note.title, "content": note.content };
    collection.insert_one(bson_doc).await.unwrap();
    Json("✅ Document added successfully")
}

// Get all documents from selected collection
async fn get_documents(
    Path(collection_name): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<Note>> {
    let collection = state.db.collection::<Document>(&collection_name);
    let mut cursor = collection.find(doc! {}).await.unwrap();
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
