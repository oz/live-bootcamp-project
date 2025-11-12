use auth_service::utils::constants::JWT_COOKIE_NAME;

use crate::helpers::{TestApp, get_random_email};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let body = serde_json::json!({ "invalid": "input", });

    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 422);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let mut app = TestApp::new().await;

    // Create a user
    let random_email = get_random_email();
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    // Login to get a token
    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    // Verify the token we just received...
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());

    let body = serde_json::json!({ "token": auth_cookie.value(), });
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 200);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;
    let body = serde_json::json!({ "token": "does not exist", });
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 401);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let mut app = TestApp::new().await;
    let mut banned_store = app.banned_tokens_store.write().await;
    assert!(banned_store.add_token("banned").await.is_ok());
    drop(banned_store); // release write lock.

    let body = serde_json::json!({ "token": "banned", });
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status().as_u16(), 401);
    app.clean_up().await;
}
