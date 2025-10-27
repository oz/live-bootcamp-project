use std::collections::HashSet;

use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};

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
        assert_eq!(store.has_token("test").await, Ok(false));

        store.tokens.insert("test".to_owned());
        assert_eq!(store.has_token("test").await, Ok(true));
    }
}
