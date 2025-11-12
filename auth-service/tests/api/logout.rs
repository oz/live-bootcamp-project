use auth_service::{ErrorResponse, utils::constants::JWT_COOKIE_NAME};
use reqwest::Url;
use secrecy::Secret;

use crate::helpers::{TestApp, get_random_email};

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let mut app = TestApp::new().await;
    // let response = app.post_logout().await;
    // assert_eq!(response.status().as_u16(), 400);

    // app.clean_up().await;
    let response = app.post_logout().await;

    assert_eq!(
        response.status().as_u16(),
        400,
        "The API did not return a 400 BAD REQUEST",
    );

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME);
    assert!(auth_cookie.is_none());
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Missing token".to_owned()
    );

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let token = app.authenticate_user(&email);

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);

    // On logout, the token is added to our banned store.
    let store = app.banned_tokens_store.read().await;
    let found = store.has_token(Secret::new(token)).await;
    assert!(found.is_ok() && found.unwrap());
    drop(store);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    app.authenticate_user(&email);

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 200);

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 400);

    app.clean_up().await;
}
