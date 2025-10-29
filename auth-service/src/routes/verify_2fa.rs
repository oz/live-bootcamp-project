use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
    utils,
};

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Parse request
    let Ok(email) = Email::parse(&request.email) else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };
    let Ok(login_attempt_id) = LoginAttemptId::parse(request.login_attempt_id) else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };
    let Ok(two_fa_code) = TwoFACode::parse(request.two_fa_code) else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };

    // Validate code
    let mut two_fa_code_store = state.two_fa_code_store.write().await;
    let Ok(code_tuple) = two_fa_code_store.get_code(&email).await else {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    };
    if (login_attempt_id, two_fa_code) != code_tuple {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }
    // Delete valid code after first use.
    if two_fa_code_store.remove_code(&email).await.is_err() {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    // Update cookie
    let Ok(auth_cookie) = utils::auth::generate_auth_cookie(&email) else {
        return (jar, Err(AuthAPIError::UnexpectedError));
    };
    let updated_jar = jar.add(auth_cookie);

    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub email: String,

    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,

    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}
