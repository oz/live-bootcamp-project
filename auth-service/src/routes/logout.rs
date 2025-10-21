use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{domain::AuthAPIError, utils};

pub async fn logout(jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let Some(cookie) = jar.get(utils::constants::JWT_COOKIE_NAME) else {
        return (jar, Err(AuthAPIError::MissingToken));
    };

    match utils::auth::validate_token(cookie.value()).await {
        Ok(_) => {
            let jar = jar.remove(utils::constants::JWT_COOKIE_NAME);
            (jar, Ok(StatusCode::OK))
        }
        Err(_) => (jar, Err(AuthAPIError::InvalidToken)),
    }
}
