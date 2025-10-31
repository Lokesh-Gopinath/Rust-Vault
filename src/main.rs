use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use futures_util::StreamExt;
use mongodb::{bson::{doc, Document}, Client, Database};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DataEntry {
    name: String,
    value: String,
}

// ✅ Serve index.html
async fn index() -> impl IntoResponse {
    Html(include_str!("web/index.html"))
}

// ✅ Add data to MongoDB
async fn add_data(
    Path(collection): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<DataEntry>,
) -> impl IntoResponse {
    let collection_ref = state.db.collection::<Document>(&collection);
    let doc = doc! { "name": payload.name, "value": payload.value };

    match collection_ref.insert_one(doc, None).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

// ✅ Get all data from MongoDB
async fn get_data(
    Path(collection): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let collection_ref = state.db.collection::<Document>(&collection);
    let mut cursor = match collection_ref.find(None, None).await {
        Ok(c) => c,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };

    let mut results = Vec::new();
    while let Some(Ok(doc)) = cursor.next().await {
        results.push(doc);
    }

    Json(results)
}

#[tokio::main]
async fn main() {
    // ✅ MongoDB setup
    let client = Client::with_uri_str("your-mongodb-uri-here")
        .await
        .expect("Failed to connect to MongoDB");
    let db = client.database("rustvault");
    let state = AppState { db: Arc::new(db) };

    // ✅ Allow CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ✅ Serve static assets from src/web
    let static_files = ServeDir::new("src/web");

    // ✅ Router setup
    let app = Router::new()
        .route("/", get(index))
        .route("/add/:collection", post(add_data))
        .route("/get/:collection", get(get_data))
        .nest_service("/static", static_files)
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🚀 Server running at http://{addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
