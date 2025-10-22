use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::{domain::AuthAPIError, utils};

pub async fn verify_token(Json(request): Json<VerifyTokenRequest>) -> impl IntoResponse {
    match utils::auth::validate_token(&request.token).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
