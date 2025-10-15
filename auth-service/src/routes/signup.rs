use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{data_stores::UserStoreError, password::Password, email::Email, AuthAPIError, User}, AppState
};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {

    let email = Email::parse(&request.email)?;
    let password = Password::parse(&request.password)?;
    let user = User::new(email, password, request.requires_2fa);
    let mut user_store = state.user_store.write().await;

    if let Err(err) = user_store.add_user(user).await && err == UserStoreError::UserAlreadyExists {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SignupResponse {
    pub message: String,
}
