use std::collections::HashSet;

use crate::domain::data_stores::BannedTokenStore;

// Hashset store for banned user tokens.
#[derive(Default)]
pub struct HashsetBannedTokenStore {
    pub tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: &str) -> bool {
        self.tokens.insert(token.to_owned())
    }

    async fn has_token(&self, token: &str) -> bool {
        self.tokens.contains(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_token() {
        let mut store = HashsetBannedTokenStore::default();
        assert!(store.add_token("test").await);
        assert!(!store.add_token("test").await);
    }

    #[tokio::test]
    async fn test_has_token() {
        let mut store = HashsetBannedTokenStore::default();
        assert!(!store.has_token("test").await);

        store.add_token("test").await;
        assert!(store.has_token("test").await);
    }
}
