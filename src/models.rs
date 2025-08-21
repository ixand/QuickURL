use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
    pub title: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreateUrlResponse {
    pub id: String,
    pub token: String,
    pub original_url: String,
    pub short_url: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub click_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UrlInfo {
    pub id: String,
    pub token: String,
    pub original_url: String,
    pub short_url: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub click_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ListUrlsResponse {
    pub urls: Vec<UrlInfo>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}
