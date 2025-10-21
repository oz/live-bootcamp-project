use super::email;
use super::password;

pub enum AuthAPIError {
    UserAlreadyExists,
    InvalidCredentials,
    IncorrectCredentials,
    UnexpectedError,

    // JWT
    MissingToken,
    InvalidToken,
}

impl From<email::EmailError> for AuthAPIError {
    fn from(_value: email::EmailError) -> Self {
        AuthAPIError::InvalidCredentials
    }
}

impl From<password::PasswordError> for AuthAPIError {
    fn from(_value: password::PasswordError) -> Self {
        AuthAPIError::InvalidCredentials
    }
}
