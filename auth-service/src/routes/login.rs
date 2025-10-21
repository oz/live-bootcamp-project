use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::utils;
use crate::{
    AppState,
    domain::{AuthAPIError, email::Email, password::Password},
};

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = Email::parse(&request.email);
    if email.is_err() {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }
    let email = email.unwrap();

    let password = Password::parse(&request.password);
    if password.is_err() {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }
    let password = password.unwrap();

    let user_store = &state.user_store.read().await;
    let user = user_store.get_user(email).await;
    match user {
        Ok(user) if user.password == password => {
            let auth_cookie = utils::auth::generate_auth_cookie(&(user.email));
            if auth_cookie.is_err() {
                return (jar, Err(AuthAPIError::UnexpectedError));
            }
            let auth_cookie = auth_cookie.unwrap();
            let updated_jar = jar.add(auth_cookie);
            (updated_jar, Ok(StatusCode::OK.into_response()))
        }
        _ => (jar, Err(AuthAPIError::IncorrectCredentials)),
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
