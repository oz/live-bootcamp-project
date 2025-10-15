use std::collections::HashMap;

use crate::domain::data_stores::{UserStore, UserStoreError};
use crate::domain::email::Email;
use crate::domain::password::Password;
use crate::domain::User;

// Store users in a HashMap (in memory) for now.
#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
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

    async fn get_user(&self, email: Email) -> Result<&User, UserStoreError> {
        self.users.get(&email).ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: Email, password: Password) -> Result<(), UserStoreError> {
        match self.users.get(&email) {
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

        let user = User::new(
            Email::parse("a@example.com").unwrap(),
            Password::parse("password123").unwrap(),
            false,
        );
        let duplicate = User::new(
            Email::parse("a@example.com").unwrap(),
            Password::parse("password123").unwrap(),
            false,
        );

        assert_eq!(store.add_user(user).await, Ok(()));
        assert_eq!(
            store.add_user(duplicate).await,
            Err(UserStoreError::UserAlreadyExists)
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let email = "a@example.com";
        let password = "password123";

        let mut store = HashmapUserStore::default();

        let result = store.get_user(Email::parse(email).unwrap()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserNotFound);

        let sample = User::new(
            Email::parse(email).unwrap(),
            Password::parse(password).unwrap(),
            false,
        );
        assert_eq!(store.add_user(sample).await, Ok(()));

        let result = store.get_user(Email::parse(email).unwrap()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email.as_ref(), email);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let email = "a@example.com";
        let password = "good-password";
        let valid_user = User::new(
            Email::parse(email).unwrap(),
            Password::parse(password).unwrap(),
            false,
        );
        assert!(store.add_user(valid_user).await.is_ok());

        // ok
        assert!(store
            .validate_user(
                Email::parse(email).unwrap(),
                Password::parse(password).unwrap()
            )
            .await
            .is_ok());

        // bad password
        let result = store
            .validate_user(
                Email::parse(email).unwrap(),
                Password::parse("bad password").unwrap(),
            )
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::InvalidCredentials);

        // bad everything
        let result = store
            .validate_user(
                Email::parse("bad@example.com").unwrap(),
                Password::parse("bad password").unwrap(),
            )
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserNotFound);
    }
}
