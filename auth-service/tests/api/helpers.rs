use fake::Fake;
use fake::faker::internet::en::{Password, SafeEmail};
use reqwest::{Url, cookie::Jar};
use sqlx::Executor;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use auth_service::{
    Application,
    app_state::{AppState, BannedTokenStoreType, TwoFACodeStoreType},
    domain::email::Email,
    get_postgres_pool,
    services::{
        PostgresUserStore, hashmap_two_fa_code_store::HashmapTwoFACodeStore,
        hashset_banned_token_store::HashsetBannedTokenStore, mock_email_client::MockEmailClient,
    },
    utils::{self, constants::JWT_COOKIE_NAME},
};
use utils::constants::DATABASE_URL;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_tokens_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
}

impl TestApp {
    pub async fn new() -> Self {
        let pg_pool = configure_postgresql().await;

        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_tokens_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        let email_client = Arc::new(RwLock::new(MockEmailClient {}));
        let cookie_jar = Arc::new(Jar::default());
        let app_state = AppState {
            user_store,
            banned_tokens_store: banned_tokens_store.clone(),
            two_fa_code_store: two_fa_code_store.clone(),
            email_client: email_client.clone(),
        };
        let app = Application::build(app_state, utils::constants::test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        // Create new `TestApp` instance and return it
        Self {
            address,
            cookie_jar,
            http_client,
            banned_tokens_store,
            two_fa_code_store,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn authenticate_user(&self, email: &str) -> String {
        let email = Email::parse(email).unwrap();
        let token = utils::auth::generate_auth_cookie(&email).unwrap();
        self.cookie_jar.add_cookie_str(
            &format!(
                "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
                JWT_COOKIE_NAME,
                token.value()
            ),
            &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
        );
        token.value().to_owned()
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_account(&self, email: &str, password: &str, requires2fa: bool) -> bool {
        let signup_body = serde_json::json!({
            "email": email,
            "password": password,
            "requires2FA": requires2fa,
        });
        let response = self.post_signup(&signup_body).await;
        response.status().as_u16() == 201
    }
}

pub fn get_random_email() -> String {
    SafeEmail().fake()
}

pub fn get_random_password() -> String {
    Password(10..15).fake()
}

async fn configure_postgresql() -> PgPool {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}
