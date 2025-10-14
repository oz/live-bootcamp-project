use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

// Store users in a HashMap (in memory) for now.
#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    // TODO: Implement a public method called `get_user`, which takes an
    // immutable reference to self and an email string slice as arguments.
    // This function should return a `Result` type containing either a
    // `User` object or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        // if let Some(user) = self.users.get(email) {
        //     Ok(user)
        // } else {
        //     Err(UserStoreError::UserNotFound)
        // }
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) if user.password == password => Ok(()),
            Some(_user) => Err(UserStoreError::InvalidCredentials),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    // TODO: Implement a public method called `validate_user`, which takes an
    // immutable reference to self, an email string slice, and a password string slice
    // as arguments. `validate_user` should return a `Result` type containing either a
    // unit type `()` if the email/password passed in match an existing user, or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    // Return `UserStoreError::InvalidCredentials` if the password is incorrect.
}

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("a@example.com", "password123", false);
        let duplicate = User::new("a@example.com", "password123", false);

        assert_eq!(store.add_user(user), Ok(()));
        assert_eq!(
            store.add_user(duplicate),
            Err(UserStoreError::UserAlreadyExists)
        );
    }

    #[tokio::test]
    async fn test_get_user() {
        let email = "a@example.com";
        let mut store = HashmapUserStore::default();
        let result = store.get_user(email);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserNotFound);

        let sample = User::new(email, "password123", false);
        assert_eq!(store.add_user(sample), Ok(()));

        let result = store.get_user(email);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, email);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let email = "a@example.com";
        let password = "good-password";
        let valid_user = User::new(email, password, false);
        assert!(store.add_user(valid_user).is_ok());

        // ok
        assert!(store.validate_user(email, password).is_ok());

        // bad password
        let result = store.validate_user(email, "bad-password");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::InvalidCredentials);

        // bad everything
        let result = store.validate_user("bad-user", "bad-password");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UserStoreError::UserNotFound);
    }
}
