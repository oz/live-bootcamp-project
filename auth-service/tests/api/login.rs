use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "test@example.com",
        "password": false,
    });

    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "not-an-email",
        "password": "password123",
    });

    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let body = serde_json::json!({
        "email": "does-not-exist@example.com",
        "password": "password123",
    });

    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 401);
}
