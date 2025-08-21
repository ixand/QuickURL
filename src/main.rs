use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Redirect},
    routing::{get, post},
    Router,
};
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

mod models;
mod token;

use models::*;
use token::TokenGenerator;

#[derive(Clone)]
pub struct AppState {
    db: SqlitePool,
    token_gen: TokenGenerator,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Starting QuickURL API server...");

    // Initialize database
let database_url = "sqlite:quickurl.db";
let db = SqlitePool::connect(database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState {
        db,
        token_gen: TokenGenerator::new(),
    };

    // Build the application with routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/shorten", post(create_short_url))
        .route("/urls", get(list_urls))
        .route("/urls/:token", get(get_url_info))
        .route("/urls/:token", axum::routing::delete(delete_url))
        .route("/:token", get(redirect_url))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("📡 Server running on http://0.0.0.0:3000");
    println!("📚 API Endpoints:");
    println!("  POST /shorten - Create short URL");
    println!("  GET  /urls - List all URLs");
    println!("  GET  /urls/:token - Get URL info");
    println!("  DELETE /urls/:token - Delete URL");
    println!("  GET  /:token - Redirect to original URL");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "QuickURL".to_string(),
        version: "0.1.0".to_string(),
    })
}

async fn create_short_url(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUrlRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate URL
    if !payload.url.starts_with("http://") && !payload.url.starts_with("https://") {
        return Err(AppError::BadRequest("URL must start with http:// or https://".into()));
    }

    // Generate unique token
    let token = state.token_gen.generate();
    let id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now();
    let expires_at = payload.expires_at.unwrap_or_else(|| {
        chrono::Utc::now() + chrono::Duration::days(30) // Default 30 days
    });

    // Insert into database
    sqlx::query(
        r#"
        INSERT INTO urls (id, token, original_url, title, created_at, expires_at, click_count)
        VALUES (?, ?, ?, ?, ?, ?, 0)
        "#
    )
    .bind(&id)
    .bind(&token)
    .bind(&payload.url)
    .bind(&payload.title)
    .bind(created_at)
    .bind(expires_at)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let response = CreateUrlResponse {
        id,
        token: token.clone(),
        original_url: payload.url,
        short_url: format!("http://localhost:3000/{}", token),
        title: payload.title,
        created_at,
        expires_at,
        click_count: 0,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn list_urls(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query("SELECT * FROM urls ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let urls: Vec<UrlInfo> = rows
        .into_iter()
        .map(|row| UrlInfo {
            id: row.get("id"),
            token: row.get("token"),
            original_url: row.get("original_url"),
            short_url: format!("http://localhost:3000/{}", row.get::<String, _>("token")),
            title: row.get("title"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
            click_count: row.get("click_count"),
        })
        .collect();

    Ok(Json(ListUrlsResponse { urls }))
}

async fn get_url_info(
    Path(token): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query("SELECT * FROM urls WHERE token = ?")
        .bind(&token)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    match row {
        Some(row) => {
            let url_info = UrlInfo {
                id: row.get("id"),
                token: row.get("token"),
                original_url: row.get("original_url"),
                short_url: format!("http://localhost:3000/{}", token),
                title: row.get("title"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
                click_count: row.get("click_count"),
            };
            Ok(Json(url_info))
        }
        None => Err(AppError::NotFound("URL not found".into())),
    }
}

async fn delete_url(
    Path(token): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query("DELETE FROM urls WHERE token = ?")
        .bind(&token)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("URL not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn redirect_url(
    Path(token): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // Get URL and check if exists and not expired
    let row = sqlx::query("SELECT * FROM urls WHERE token = ?")
        .bind(&token)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    match row {
        Some(row) => {
            let expires_at: chrono::DateTime<chrono::Utc> = row.get("expires_at");
            
            // Check if expired
            if chrono::Utc::now() > expires_at {
                return Err(AppError::Gone("URL has expired".into()));
            }

            let original_url: String = row.get("original_url");

            // Increment click count
            sqlx::query("UPDATE urls SET click_count = click_count + 1 WHERE token = ?")
                .bind(&token)
                .execute(&state.db)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            Ok(Redirect::permanent(&original_url))
        }
        None => Err(AppError::NotFound("URL not found".into())),
    }
}

#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    NotFound(String),
    BadRequest(String),
    Gone(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Gone(msg) => (StatusCode::GONE, msg),
        };

        let body = Json(serde_json::json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
