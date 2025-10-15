use validator::ValidateEmail;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Email(String);

#[derive(Debug)]
pub enum EmailError {
    InvalidEmail,
}

impl Email {
    pub fn parse(email: &str) -> Result<Email, EmailError> {
        if email.validate_email() {
            Ok(Email(email.to_owned()))
        } else {
            Err(EmailError::InvalidEmail)
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Fake, faker::internet::en::*};

    #[test]
    fn test_parse_email() {
        let valid_email: String = FreeEmail().fake();
        let invalid_email = "not an email";

        assert!(Email::parse(&valid_email).is_ok());
        assert!(Email::parse(invalid_email).is_err());
    }

    #[test]
    fn test_as_ref() {
        let email = Email::parse("valid@example.com");
        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_ref(), "valid@example.com");
    }
}
