//use crate::helpers::{TestApp, get_random_email, get_random_password};
//use auth_service::{
//    domain::email::Email, routes::TwoFactorAuthResponse, utils::constants::JWT_COOKIE_NAME,
//};
//
//#[tokio::test]
//async fn should_return_422_if_malformed_credentials() {
//    let mut app = TestApp::new().await;
//    let body = serde_json::json!({
//        "email": "test@example.com",
//        "password": false,
//    });
//
//    let response = app.post_login(&body).await;
//    assert_eq!(response.status().as_u16(), 422);
//    app.clean_up().await;
//}
//
//#[tokio::test]
//async fn should_return_400_if_invalid_input() {
//    let mut app = TestApp::new().await;
//    let body = serde_json::json!({
//        "email": "not-an-email",
//        "password": "password123",
//    });
//
//    let response = app.post_login(&body).await;
//    assert_eq!(response.status().as_u16(), 400);
//    app.clean_up().await;
//}
//
//#[tokio::test]
//async fn should_return_401_if_incorrect_credentials() {
//    let mut app = TestApp::new().await;
//    let body = serde_json::json!({
//        "email": "does-not-exist@example.com",
//        "password": "password123",
//    });
//
//    let response = app.post_login(&body).await;
//    assert_eq!(response.status().as_u16(), 401);
//    app.clean_up().await;
//}
//
//#[tokio::test]
//async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
//    let mut app = TestApp::new().await;
//    let email = get_random_email();
//    let password = get_random_password();
//
//    let signup_body = serde_json::json!({
//        "email": email,
//        "password": password,
//        "requires2FA": false
//    });
//    let response = app.post_signup(&signup_body).await;
//    assert_eq!(response.status().as_u16(), 201);
//
//    let login_body = serde_json::json!({
//        "email": email,
//        "password": password,
//    });
//    let response = app.post_login(&login_body).await;
//    assert_eq!(response.status().as_u16(), 200);
//
//    let auth_cookie = response
//        .cookies()
//        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
//        .expect("No auth cookie found");
//    assert!(!auth_cookie.value().is_empty());
//    app.clean_up().await;
//}
//
//#[tokio::test]
//async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
//    let mut app = TestApp::new().await;
//    let email = get_random_email();
//    let password = get_random_password();
//
//    // Create an account with 2FA enabled.
//    let signup_body = serde_json::json!({
//        "email": email,
//        "password": password,
//        "requires2FA": true
//    });
//    let response = app.post_signup(&signup_body).await;
//    assert_eq!(response.status().as_u16(), 201);
//
//    // Login should return a 206
//    let login_body = serde_json::json!({ "email": email, "password": password });
//    let response = app.post_login(&login_body).await;
//    assert_eq!(response.status().as_u16(), 206);
//
//    let json_body = response
//        .json::<TwoFactorAuthResponse>()
//        .await
//        .expect("Could not deserialize response body to TwoFactorAuthResponse");
//    assert_eq!(json_body.message, "2FA required".to_owned());
//
//    let email = Email::parse(&email).expect("Email parse error");
//    let two_fa_code_store = app.two_fa_code_store.read().await;
//    let code = two_fa_code_store
//        .get_code(&email)
//        .await
//        .expect("login code not found");
//    assert_eq!(code.0.as_ref(), json_body.login_attempt_id);
//    drop(two_fa_code_store);
//
//    app.clean_up().await;
//}
