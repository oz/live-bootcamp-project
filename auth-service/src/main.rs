use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::{
    Application,
    app_state::AppState,
    get_postgres_pool, get_redis_client,
    services::{
        PostgresUserStore, RedisBannedTokenStore, RedisTwoFACodeStore,
        mock_email_client::MockEmailClient,
    },
    utils::constants::{DATABASE_URL, REDIS_HOST_NAME, prod},
};

#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;
    let banned_token_redis_conn = Arc::new(RwLock::new(configure_redis()));
    let two_fa_redis_conn = Arc::new(RwLock::new(configure_redis()));

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let banned_tokens_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(
        banned_token_redis_conn,
    )));
    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(two_fa_redis_conn)));
    let email_client = Arc::new(RwLock::new(MockEmailClient {}));

    let app_state = AppState {
        user_store,
        banned_tokens_store,
        two_fa_code_store,
        email_client,
    };

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

pub async fn configure_postgresql() -> PgPool {
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations at startup.
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
