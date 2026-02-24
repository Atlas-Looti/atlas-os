use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;
use dotenvy::dotenv;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiKey {
    id: Uuid,
    user_id: String,
    name: String,
    prefix: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct CreateApiKeyRequest {
    user_id: String,
    name: String,
}

#[derive(Serialize)]
struct CreateApiKeyResponse {
    key: String, // The full key (only shown once)
    record: ApiKey,
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load environment variables from .env file optionally

    // Database connection string
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Initialize Postgres Connection Pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres.");

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.expect("Failed to run migrations");

    let state = AppState { db: pool };

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any) // For development. Lock this down in production.
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the router
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/keys", get(list_keys).post(create_key))
        .route("/api/keys/:id", delete(delete_key))
        .layer(cors)
        .with_state(state);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler: List API Keys for a User
async fn list_keys(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<ApiKey>>, StatusCode> {
    let user_id = params.get("user_id").ok_or(StatusCode::BAD_REQUEST)?;

    let keys = sqlx::query_as!(
        ApiKey,
        r#"
        SELECT id, user_id, name, prefix, created_at
        FROM api_keys
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(keys))
}

// Handler: Create a new API Key
async fn create_key(
    State(state): State<AppState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, StatusCode> {
    // Generate a secure random API key starting with "atl_"
    use rand::RngCore;
    let mut random_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut random_bytes);
    
    // Hex encode the bytes
    let token = hex::encode(random_bytes);
    let full_key = format!("atl_{}", token);
    
    let prefix = full_key[0..8].to_string(); // Store just the prefix (atl_xxxx)
    
    // In a production scenario, you must HASH `full_key` before storing it!
    // For this example, we'll store a placeholder hash string.
    let key_hash = format!("hashed_{}", full_key); 

    let new_key = sqlx::query_as!(
        ApiKey,
        r#"
        INSERT INTO api_keys (user_id, name, prefix, key_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, name, prefix, created_at
        "#,
        payload.user_id,
        payload.name,
        prefix,
        key_hash
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        println!("Error inserting: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(CreateApiKeyResponse {
        key: full_key,
        record: new_key,
    }))
}

// Handler: Delete an API Key
async fn delete_key(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query!(
        r#"
        DELETE FROM api_keys
        WHERE id = $1
        "#,
        id
    )
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
