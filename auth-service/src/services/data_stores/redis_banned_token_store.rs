use std::sync::Arc;

use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let key = get_key(token);
        self.conn
            .write()
            .await
            .set_ex::<_, _, String>(key, true, TOKEN_TTL_SECONDS as u64)
            .map_err(|err| {
                println!("error: add_token (set_ex) {:?}", err);
                BannedTokenStoreError::UnexpectedError
            })?;

        Ok(())
    }

    async fn has_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);
        let exists: bool = self.conn.write().await.exists(key).map_err(|err| {
            println!("error: add_token (set_ex) {:?}", err);
            BannedTokenStoreError::UnexpectedError
        })?;

        Ok(exists)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
