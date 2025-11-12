use color_eyre::eyre::{Result, eyre};
use validator::validate_email;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Email(String);

impl Email {
    pub fn parse(s: String) -> Result<Email> {
        if validate_email(&s) {
            Ok(Email(s))
        } else {
            Err(eyre!("{} is not a valid email.", s))
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
        let invalid_email = String::from("not an email");

        assert!(Email::parse(valid_email).is_ok());
        assert!(Email::parse(invalid_email).is_err());
    }

    #[test]
    fn test_as_ref() {
        let email = Email::parse("valid@example.com".to_owned());
        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_ref(), "valid@example.com");
    }
}
