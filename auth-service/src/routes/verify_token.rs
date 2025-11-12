use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::{AppState, domain::AuthAPIError, utils};

#[tracing::instrument(name = "Verify token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    match utils::auth::validate_token(state.banned_tokens_store, &request.token).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
