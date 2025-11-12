use crate::{
    AppState,
    domain::{AuthAPIError, User, email::Email, password::Password},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use tracing;

#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let password =
        Password::parse(request.password.clone()).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let user = User::new(email, password, request.requires_2fa);
    let mut user_store = state.user_store.write().await;

    if user_store.get_user(user.email.clone()).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    if let Err(e) = user_store.add_user(user).await {
        return Err(AuthAPIError::UnexpectedError(e.into()));
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SignupResponse {
    pub message: String,
}
