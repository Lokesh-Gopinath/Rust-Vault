use axum::{
    extract::{Path, State},
    response::{Html, Json},
    routing::{delete, get, post},
    Router,
};
use futures::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Client, Database,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use axum::serve;

/* ---------- models ---------- */
#[derive(Debug, Serialize, Deserialize)]
struct Note {
    title: String,
    content: String,
}

/* ---------- state ---------- */
#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

/* ---------- main ---------- */
#[tokio::main]
async fn main() {
    let mongo_uri = std::env::var("MONGO_URI").expect("MONGO_URI must be set");
    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");
    let db = Arc::new(client.database("rustvault"));

    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/collections", get(get_collections))
        .route("/documents/:collection", get(get_documents))
        .route("/add/:collection", post(add_document))
        .route("/delete/:collection/:id", delete(delete_document)) // NEW
        .with_state(AppState { db: db.clone() })
        .layer(ServiceBuilder::new().layer(cors));

    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind port 8080");
    println!("✅ RustVault running on http://0.0.0.0:8080");
    serve(listener, app).await.unwrap();
}

/* ---------- handlers ---------- */
async fn root() -> Html<&'static str> {
    Html(include_str!("web/index.html"))
}

async fn get_collections(State(state): State<AppState>) -> Json<Vec<String>> {
    let names = state.db.list_collection_names().await.unwrap_or_default();
    Json(names)
}

async fn get_documents(
    Path(collection_name): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<serde_json::Value>> {
    let coll = state.db.collection::<Document>(&collection_name);
    let mut cursor = coll.find(doc! {}).await.unwrap();
    let mut docs = Vec::new();

    while let Some(res) = cursor.next().await {
        if let Ok(doc) = res {
            docs.push(serde_json::to_value(&doc).unwrap());
        }
    }
    Json(docs)
}

async fn add_document(
    Path(collection_name): Path<String>,
    State(state): State<AppState>,
    Json(note): Json<Note>,
) -> Json<&'static str> {
    let coll = state.db.collection::<Document>(&collection_name);
    let new_doc = doc! { "title": note.title, "content": note.content };
    coll.insert_one(new_doc).await.unwrap();
    Json("✅ Document added")
}

async fn delete_document(
    Path((collection_name, id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Json<&'static str> {
    let coll = state.db.collection::<Document>(&collection_name);
    let oid = ObjectId::parse_str(&id).expect("Invalid ObjectId");
    coll.delete_one(doc! { "_id": oid }).await.unwrap();
    Json("✅ Document deleted")
}

