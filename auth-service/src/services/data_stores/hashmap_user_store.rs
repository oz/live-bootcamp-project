use std::collections::HashMap;

use crate::domain::User;
use crate::domain::data_stores::{UserStore, UserStoreError};
use crate::domain::email::Email;
use crate::domain::password::Password;

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

    async fn get_user(&self, email: Email) -> Result<User, UserStoreError> {
        match self.users.get(&email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
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
    use secrecy::Secret;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();

        let user = User::new(
            Email::parse(Secret::from("a@example.com".to_owned())).unwrap(),
            Password::parse(Secret::new("password123".to_owned())).unwrap(),
            false,
        );
        let duplicate = User::new(
            Email::parse(Secret::from("a@example.com".to_owned())).unwrap(),
            Password::parse(Secret::new("password123".to_owned())).unwrap(),
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
        let email =
            Email::parse(Secret::from("a@example.com".to_owned())).expect("Invalid test email");
        let password =
            Password::parse(Secret::new("password123".to_owned())).expect("Invalid test password");

        let mut store = HashmapUserStore::default();

        let result = store.get_user(email.clone()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserNotFound);

        let sample = User::new(email.clone(), password, false);
        assert_eq!(store.add_user(sample).await, Ok(()));

        let result = store.get_user(email.clone()).await;
        assert!(result.is_ok());
        let result_email = result.unwrap().email;
        assert_eq!(result_email, email);
    }

    #[tokio::test]
    async fn test_validate_user() {
        // Store a valid user.
        let mut store = HashmapUserStore::default();
        let email =
            Email::parse(Secret::from("a@example.com".to_owned())).expect("Invalid test email");
        let password = Password::parse(Secret::new("good-password".to_owned()))
            .expect("Invalid test password");
        let valid_user = User::new(email.clone(), password.clone(), false);
        assert!(store.add_user(valid_user).await.is_ok());

        // ok
        assert!(store.validate_user(email.clone(), password).await.is_ok());

        // bad password
        let incorrect_password =
            Password::parse(Secret::new("bad password".to_owned())).expect("Invalid test password");
        let result = store.validate_user(email, incorrect_password.clone()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::InvalidCredentials);

        // bad everything
        let incorrect_user =
            Email::parse(Secret::from("bad@example.com".to_owned())).expect("Invalid test email");
        let result = store
            .validate_user(incorrect_user, incorrect_password)
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserNotFound);
    }
}
