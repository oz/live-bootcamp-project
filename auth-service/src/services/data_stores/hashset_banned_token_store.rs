use secrecy::{ExposeSecret, Secret};

use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};
use std::collections::HashSet;

// Hashset store for banned user tokens.
#[derive(Default)]
pub struct HashsetBannedTokenStore {
    pub tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        let secret = token.expose_secret();
        self.tokens.insert(secret.clone());
        Ok(())
    }

    async fn has_token(&self, token: Secret<String>) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token.expose_secret()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token() {
        let mut store = HashsetBannedTokenStore::default();
        assert!(
            store
                .add_token(Secret::new("test".to_owned()))
                .await
                .is_ok()
        );
        assert!(store.tokens.contains("test"));
    }

    #[tokio::test]
    async fn test_has_token() {
        let mut store = HashsetBannedTokenStore::default();
        let found = store.has_token(Secret::new("test".to_owned())).await;
        assert_eq!(found.unwrap(), false);

        store.tokens.insert("test".to_owned());
        let found = store.has_token(Secret::new("test".to_owned())).await;
        assert_eq!(found.unwrap(), true);
    }
}
