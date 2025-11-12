use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        // match self.codes.insert(email, (login_attempt_id, code)) {
        //     None => Ok(()),
        //     _ => Err(TwoFACodeStoreError::UnexpectedError),
        // }
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes
            .remove(email)
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)?;
        Ok(())
    }
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let code = self
            .codes
            .get(email)
            .ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)?;
        Ok(code.clone())
    }
}

#[cfg(test)]
mod tests {
    use secrecy::Secret;

    use super::*;

    #[tokio::test]
    async fn test_add_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let email =
            Email::parse(Secret::new("test@example.com".to_owned())).expect("invalid email");
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store
            .add_code(email.clone(), login_attempt_id, code)
            .await
            .expect("add code failed");
        assert!(store.codes.contains_key(&email));
    }

    #[tokio::test]
    async fn test_remove_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let email =
            Email::parse(Secret::new("test@example.com".to_owned())).expect("invalid email");
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store
            .add_code(email.clone(), login_attempt_id, code)
            .await
            .unwrap();
        store.remove_code(&email).await.unwrap();
        assert!(!store.codes.contains_key(&email));
    }

    #[tokio::test]
    async fn test_get_code() {
        let mut store = HashmapTwoFACodeStore::default();
        let email =
            Email::parse(Secret::new("test@example.com".to_owned())).expect("invalid email");
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await
            .unwrap();
        let (id, c) = store.get_code(&email).await.unwrap();
        assert_eq!(id, login_attempt_id);
        assert_eq!(c, code);
    }
}
