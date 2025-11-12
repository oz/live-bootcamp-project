use crate::{
    AppState,
    domain::AuthAPIError,
    utils::{self, constants::JWT_COOKIE_NAME},
};
use axum::{extract::State, http::StatusCode};
use axum_extra::extract::{CookieJar, cookie};
use secrecy::Secret;
use tracing;

#[tracing::instrument(name = "Logout", skip_all)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<StatusCode, AuthAPIError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };
    let token = Secret::new(cookie.value().to_owned());

    // Validate token
    let _ = match utils::auth::validate_token(state.banned_tokens_store.clone(), &token).await {
        Ok(claims) => claims,
        Err(_) => return (jar, Err(AuthAPIError::InvalidToken)),
    };

    // Add token to banned list
    if let Err(e) = state
        .banned_tokens_store
        .write()
        .await
        .add_token(token)
        .await
    {
        return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
    }

    // Remove jwt cookie
    let jar = jar.remove(cookie::Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
