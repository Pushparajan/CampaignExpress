//! Simple bearer token authentication middleware.
//!
//! Development: accepts any "admin:password" login, returns a static token.
//! Production: replace with JWT + OAuth2 (jsonwebtoken crate + Auth0/Ory).

use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{Duration, Utc};
use rand::Rng;

use crate::models::{ErrorResponse, LoginRequest, LoginResponse};

/// Hard-coded API token for development. Production: use JWT.
const DEV_TOKEN_PREFIX: &str = "ce_dev_";

/// Validate a login request and return a bearer token.
pub fn authenticate(req: &LoginRequest) -> Result<LoginResponse, String> {
    // Development: accept admin/admin or any user with password "campaign2024"
    if (req.username == "admin" && req.password == "admin") || req.password == "campaign2024" {
        let token = generate_token();
        Ok(LoginResponse {
            token,
            user: req.username.clone(),
            expires_at: Utc::now() + Duration::hours(24),
        })
    } else {
        Err("Invalid credentials".to_string())
    }
}

/// Generate a random bearer token.
fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    format!(
        "{}{}",
        DEV_TOKEN_PREFIX,
        bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    )
}

/// Axum middleware layer that checks for a valid bearer token.
/// Skips auth for login endpoint and health checks.
pub async fn auth_middleware(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();

    // Skip auth for login, health, and non-management routes
    if path.ends_with("/auth/login")
        || path.starts_with("/health")
        || path.starts_with("/ready")
        || path.starts_with("/live")
        || !path.contains("/management/")
    {
        return next.run(req).await;
    }

    // Check Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(value) if value.starts_with("Bearer ") => {
            let token = &value[7..];
            if token.starts_with(DEV_TOKEN_PREFIX) && token.len() > DEV_TOKEN_PREFIX.len() {
                next.run(req).await
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "invalid_token".to_string(),
                        message: "Invalid or expired bearer token".to_string(),
                    }),
                )
                    .into_response()
            }
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "missing_auth".to_string(),
                message: "Authorization header with Bearer token required".to_string(),
            }),
        )
            .into_response(),
    }
}
