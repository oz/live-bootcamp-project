use std::sync::Arc;

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{
    Email,
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let twofa = TwoFATuple(
            login_attempt_id.as_ref().to_owned(),
            code.as_ref().to_owned(),
        );
        let key = get_key(&email);
        let value = serde_json::to_string(&twofa).map_err(|err| {
            eprintln!("error: JSON error in add_code: {:?}", err);
            TwoFACodeStoreError::UnexpectedError
        })?;
        self.conn
            .write()
            .await
            .set_ex::<_, _, String>(key, value, TEN_MINUTES_IN_SECONDS)
            .map_err(|err| {
                println!("error: add_code (set_ex) {:?}", err);
                TwoFACodeStoreError::UnexpectedError
            })?;
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);
        self.conn.write().await.del::<_, u64>(key).map_err(|err| {
            eprintln!("error: remove_code (del) {:?}", err);
            TwoFACodeStoreError::UnexpectedError
        })?;

        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let key = get_key(&email);
        let result: String = self.conn.write().await.get(key).map_err(|err| {
            eprintln!("error: get_code (get) {:?}", err);
            TwoFACodeStoreError::LoginAttemptIdNotFound
        })?;
        let twofa: TwoFATuple = serde_json::from_str(&result).map_err(|err| {
            eprintln!("error: get_code (serde_json::from_str) {:?}", err);
            TwoFACodeStoreError::UnexpectedError
        })?;
        let login_attempt_id = LoginAttemptId::parse(twofa.0).map_err(|err| {
            eprintln!("error: parsing LoginAttemptId from Redis {:?}", err);
            TwoFACodeStoreError::UnexpectedError
        })?;
        let code = TwoFACode::parse(twofa.1).map_err(|err| {
            eprintln!("error: parsing TwoFACode from Redis {:?}", err);
            TwoFACodeStoreError::UnexpectedError
        })?;

        Ok((login_attempt_id, code))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
