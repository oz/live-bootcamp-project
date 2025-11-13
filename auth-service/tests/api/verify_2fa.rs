use crate::helpers::{TestApp, get_random_email, get_random_password};
use auth_service::{domain::Email, utils::constants::JWT_COOKIE_NAME};
use secrecy::{ExposeSecret, Secret};
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!({ "email": random_email, "loginAttemptId": "bad", "2FACode": false }),
        serde_json::json!({ "loginAttemptId": 0, "2FACode": 42 }),
        serde_json::json!({ "email": "wrong", "loginAttemptId": 0, "2FACode": 42 }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let test_cases =
        [serde_json::json!({ "email": random_email, "loginAttemptId": "bad", "2FACode": "0" })];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;

    let random_email = get_random_email();
    let incorrect_credentials = serde_json::json!({
        "email": random_email,
        "loginAttemptId": "0e57bc50-071e-4965-a60f-4f0b3137c8bb",
        "2FACode": "123456",
    });

    let response = app.post_verify_2fa(&incorrect_credentials).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Failed for input: {:?}",
        incorrect_credentials
    );

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    // This calls login twice. Then, attempt to call verify-fa with the 2FA code from the first
    // login request. This should fail with a 401 error.

    let mut app = TestApp::new().await;
    let email = get_random_email();
    let password = get_random_password();
    let email_parsed = Email::parse(Secret::new(email.clone())).expect("invalid email");

    assert!(true == app.create_account(&email, &password, true).await);
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    // Try to login, to create a first 2FA code.
    let body = serde_json::json!({ "email": email, "password": password});
    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 206);

    let two_fa_store = app.two_fa_code_store.read().await;
    let code_tuple = two_fa_store
        .get_code(&email_parsed)
        .await
        .expect("2FA codes not found");
    drop(two_fa_store);

    // Try a second login to invalidate the previous codes.
    let body = serde_json::json!({ "email": email, "password": password});
    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 206);

    // // Try to verify 2FA with the old login codes.
    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": code_tuple.0.as_ref().expose_secret(),
        "2FACode": code_tuple.1.as_ref().expose_secret(),
    });
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Failed for input: {:?}",
        body
    );

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    // Call login twice. Then, attempt to call verify-fa with the 2FA code from the first login
    // request.
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email_parsed = Email::parse(Secret::new(email.clone())).expect("invalid email");

    assert!(true == app.create_account(&email, "password123", true).await);
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Try to login, to create a first 2FA code.
    let body = serde_json::json!({ "email": email, "password": "password123" });
    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 206);

    let two_fa_store = app.two_fa_code_store.read().await;
    let code_tuple = two_fa_store
        .get_code(&email_parsed)
        .await
        .expect("2FA codes not found");
    drop(two_fa_store);

    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": code_tuple.0.as_ref().expose_secret(),
        "2FACode": code_tuple.1.as_ref().expose_secret(),
    });
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email_parsed = Email::parse(Secret::new(email.clone())).expect("invalid email");

    assert!(true == app.create_account(&email, "password123", true).await);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Try to login, to get a first 2FA code.
    let body = serde_json::json!({ "email": email, "password": "password123" });
    let response = app.post_login(&body).await;
    assert_eq!(response.status().as_u16(), 206);

    let two_fa_store = app.two_fa_code_store.read().await;
    let code_tuple = two_fa_store
        .get_code(&email_parsed)
        .await
        .expect("2FA codes not found");
    drop(two_fa_store);

    // Verifying the codes once works
    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": code_tuple.0.as_ref().expose_secret(),
        "2FACode": code_tuple.1.as_ref().expose_secret(),
    });
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());

    // Using the same code twice does not.
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status().as_u16(), 401);

    app.clean_up().await;
}
