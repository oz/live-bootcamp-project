use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use tracing;

use crate::{
    AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, Password, TwoFACode},
    utils::auth::generate_auth_cookie,
};

#[tracing::instrument(name = "Login", skip_all)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let Ok(email) = Email::parse(request.email) else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };
    let Ok(password) = Password::parse(request.password) else {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };

    let user_store = &state.user_store.read().await;
    if user_store
        .validate_user(email.clone(), password)
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = match user_store.get_user(email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    // Handle request based on user's 2FA configuration
    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

#[tracing::instrument(name = "Handle 2FA", skip_all)]
async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    // Generate a new random login attempt ID and 2FA code.
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    if let Err(e) = state
        .two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
    {
        return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
    }

    // Send code by email.
    let email_client = state.email_client.write().await;
    if let Err(err) = email_client
        .send_email(
            email,
            "Your 2FA login code",
            format!("Hi there, here is you 2FA code: {}", two_fa_code.as_ref()).as_str(),
        )
        .await
    {
        return (jar, Err(AuthAPIError::UnexpectedError(err)));
    }

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.as_ref().to_string(),
    }));

    (jar, Ok((StatusCode::PARTIAL_CONTENT, response)))
}

#[tracing::instrument(name = "Handle No 2FA", skip_all)]
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let auth_cookie = match generate_auth_cookie(email) {
        Ok(cookie) => cookie,
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))),
    };
    let updated_jar = jar.add(auth_cookie);

    (
        updated_jar,
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
    )
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
