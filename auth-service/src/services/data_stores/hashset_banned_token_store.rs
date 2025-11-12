use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};
use std::collections::HashSet;

// Hashset store for banned user tokens.
#[derive(Default)]
pub struct HashsetBannedTokenStore {
    pub tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        self.tokens.insert(token.to_owned());
        Ok(())
    }

    async fn has_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token() {
        let mut store = HashsetBannedTokenStore::default();
        assert!(store.add_token("test").await.is_ok());
        assert!(store.tokens.contains("test"));
    }

    #[tokio::test]
    async fn test_has_token() {
        let mut store = HashsetBannedTokenStore::default();
        let found = store.has_token("test").await;
        assert_eq!(found.unwrap(), false);

        store.tokens.insert("test".to_owned());
        let found = store.has_token("test").await;
        assert_eq!(found.unwrap(), true);
    }
}
