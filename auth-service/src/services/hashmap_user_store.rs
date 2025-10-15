use std::collections::HashMap;

use crate::domain::data_stores::{UserStore, UserStoreError};
use crate::domain::User;

// Store users in a HashMap (in memory) for now.
#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) if user.password == password => Ok(()),
            Some(_user) => Err(UserStoreError::InvalidCredentials),
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("a@example.com", "password123", false);
        let duplicate = User::new("a@example.com", "password123", false);

        assert_eq!(store.add_user(user).await, Ok(()));
        assert_eq!(
            store.add_user(duplicate).await,
            Err(UserStoreError::UserAlreadyExists)
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let email = "a@example.com";
        let mut store = HashmapUserStore::default();
        let result = store.get_user(email).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserNotFound);

        let sample = User::new(email, "password123", false);
        assert_eq!(store.add_user(sample).await, Ok(()));

        let result = store.get_user(email).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, email);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let email = "a@example.com";
        let password = "good-password";
        let valid_user = User::new(email, password, false);
        assert!(store.add_user(valid_user).await.is_ok());

        // ok
        assert!(store.validate_user(email, password).await.is_ok());

        // bad password
        let result = store.validate_user(email, "bad-password").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::InvalidCredentials);

        // bad everything
        let result = store.validate_user("bad-user", "bad-password").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserNotFound);
    }
}
