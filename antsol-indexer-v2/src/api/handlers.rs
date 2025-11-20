use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use deadpool_postgres::Pool;

use crate::db::{models::*, queries};
use crate::indexer::listener::{extract_ipfs_hash, ingest_event};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

pub async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("OK".to_string()))
}

pub async fn search_packages_handler(
    State(pool): State<Pool>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<Package>>>, StatusCode> {
    match queries::search_packages(&pool, &params.q, params.limit, params.offset).await {
        Ok(packages) => Ok(Json(ApiResponse::success(packages))),
        Err(e) => {
            tracing::error!("Search error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_package_handler(
    State(pool): State<Pool>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<PackageWithVersions>>, StatusCode> {
    match queries::get_package_with_versions(&pool, &name).await {
        Ok(Some(pkg)) => Ok(Json(ApiResponse::success(pkg))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Get package error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_packages_handler(
    State(pool): State<Pool>,
    Query(params): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Package>>>, StatusCode> {
    match queries::list_packages(&pool, params.limit, params.offset).await {
        Ok(packages) => Ok(Json(ApiResponse::success(packages))),
        Err(e) => {
            tracing::error!("List packages error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_stats_handler(
    State(pool): State<Pool>,
) -> Result<Json<ApiResponse<Stats>>, StatusCode> {
    match queries::get_stats(&pool).await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            tracing::error!("Get stats error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_recent_events_handler(
    State(pool): State<Pool>,
    Query(params): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Event>>>, StatusCode> {
    let limit = params.limit.min(100); // Cap at 100
    match queries::get_recent_events(&pool, limit).await {
        Ok(events) => Ok(Json(ApiResponse::success(events))),
        Err(e) => {
            tracing::error!("Get recent events error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_package_events_handler(
    State(pool): State<Pool>,
    Path(package_name): Path<String>,
    Query(params): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Event>>>, StatusCode> {
    let limit = params.limit.min(100);
    match queries::get_package_events(&pool, &package_name, limit, params.offset).await {
        Ok(events) => Ok(Json(ApiResponse::success(events))),
        Err(e) => {
            tracing::error!("Get package events error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// --- Manual ingestion endpoint for testing indexer without waiting for chain slots ---
#[derive(Deserialize)]
pub struct IngestRequest {
    pub log: String,
    pub signature: Option<String>,
    pub slot: Option<i64>,
    pub block_time: Option<i64>,
}

#[derive(Serialize)]
pub struct IngestResult {
    pub event: Option<Event>,
    pub ipfs_hash: Option<String>,
    pub message: String,
}

pub async fn ingest_log_handler(
    State(pool): State<Pool>,
    Json(req): Json<IngestRequest>,
) -> Result<Json<ApiResponse<IngestResult>>, StatusCode> {
    let signature = req.signature.unwrap_or_else(|| "manual_sig".to_string());
    let slot = req.slot.unwrap_or(0);
    let event_opt = crate::indexer::parser::parse_transaction(&req.log, &signature, slot, req.block_time);
    if let Some(event) = event_opt {
        // Store event first
        if let Err(e) = queries::insert_event(
            &pool,
            &event.event_type,
            &event.package_name,
            event.version.as_deref(),
            &event.transaction_signature,
            event.slot,
            req.block_time,
        ).await {
            tracing::warn!("Failed to insert manual event: {}", e);
        }
        // Ingest metadata
        if let Err(e) = ingest_event(&pool, &event, &req.log).await {
            tracing::warn!("Manual ingestion failed: {}", e);
        }
        let ipfs = extract_ipfs_hash(&req.log);
        Ok(Json(ApiResponse::success(IngestResult {
            event: Some(event),
            ipfs_hash: ipfs,
            message: "Event parsed and ingested".to_string(),
        })))
    } else {
        Ok(Json(ApiResponse::success(IngestResult {
            event: None,
            ipfs_hash: None,
            message: "No recognizable event in log".to_string(),
        })))
    }
}
