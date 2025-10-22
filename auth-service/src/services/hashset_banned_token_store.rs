use std::collections::HashSet;

use crate::domain::data_stores::BannedTokenStore;

// Hashset store for banned user tokens.
#[derive(Default)]
pub struct HashsetBannedTokenStore {
    pub tokens: HashSet<String>,
}

impl BannedTokenStore for HashsetBannedTokenStore {
    fn add_token(&mut self, token: &str) -> bool {
        self.tokens.insert(token.to_owned())
    }

    fn has_token(&self, token: &str) -> bool {
        self.tokens.contains(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_token() {
        let mut store = HashsetBannedTokenStore::default();
        assert!(store.add_token("test"));
        assert!(!store.add_token("test"));
    }

    #[test]
    fn test_has_token() {
        let mut store = HashsetBannedTokenStore::default();
        assert!(!store.has_token("test"));

        store.add_token("test");
        assert!(store.has_token("test"));
    }
}
