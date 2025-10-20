use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    domain::{AuthAPIError, email::Email, password::Password},
};

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&request.email)?;
    let password = Password::parse(&request.password)?;

    let user_store = &state.user_store.read().await;
    let user = user_store.get_user(email).await;
    match user {
        Ok(user) if user.password == password => Ok(StatusCode::OK.into_response()),
        _ => Err(AuthAPIError::IncorrectCredentials),
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct LoginResponse {
    pub message: String,
}
