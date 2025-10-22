use axum::{debug_handler, extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;

use crate::{AppState, domain::AuthAPIError, utils};

#[debug_handler]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<StatusCode, AuthAPIError>) {
    // Get token from cookie
    let token = jar
        .get(utils::constants::JWT_COOKIE_NAME)
        .map(|cookie| cookie.value())
        .ok_or(AuthAPIError::MissingToken);
    let token = match token {
        Ok(t) => t,
        Err(e) => return (jar, Err(e)),
    };

    // Check token
    if utils::auth::validate_token(state.banned_tokens_store.clone(), token)
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    // Try to store token as banned, but don't block logout.
    state.banned_tokens_store.write().await.add_token(token);
    let jar = jar.remove(utils::constants::JWT_COOKIE_NAME);
    (jar, Ok(StatusCode::OK))
}
