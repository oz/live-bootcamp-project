use auth_service::{ErrorResponse, routes::SignupResponse};

use crate::helpers::{TestApp, get_random_email, get_random_password};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!({ "password": "password123", "requires2FA": true }),
        serde_json::json!({ "email": random_email, "requires2FA": false }),
        serde_json::json!({ "email": "bad", "password": 123 }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let password = get_random_password();
    let valid_input =
        serde_json::json!({"email": email, "password": password, "requires2FA": false});

    let response = app.post_signup(&valid_input).await;
    assert_eq!(response.status().as_u16(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_owned(),
    };

    // Assert that we are getting the correct response body!
    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response
    );
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let test_cases = [
        serde_json::json!({ "email": "",               "password": "password123", "requires2FA": true }),
        serde_json::json!({ "email": "bad",            "password": "password123", "requires2FA": true }),
        serde_json::json!({ "email": "ok@example.com", "password": "bad",         "requires2FA": true }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let app = TestApp::new().await;

    let email = get_random_email();
    let password = get_random_password();
    let valid_input =
        serde_json::json!({"email": email, "password": password, "requires2FA": false});

    let response = app.post_signup(&valid_input).await;
    assert_eq!(response.status().as_u16(), 201);

    // Another signup with the same email
    let response = app.post_signup(&valid_input).await;
    assert_eq!(response.status().as_u16(), 409);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}
