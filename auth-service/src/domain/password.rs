const MIN_PASSWORD_LENGTH: usize = 8;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Password(String);

impl Password {
    pub fn parse(pass: String) -> Result<Password, String> {
        if pass.is_empty() || pass.len() < MIN_PASSWORD_LENGTH {
            Err("Failed to parse string to a Password type".to_owned())
        } else {
            Ok(Password(pass.to_owned()))
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert!(Password::parse("an ok password".to_owned()).is_ok());
        assert!(Password::parse("bad".to_owned()).is_err());
        assert!(Password::parse("".to_owned()).is_err());
    }

    #[test]
    fn test_as_ref() {
        let pass = Password::parse("My-Secret-Passphrase42".to_owned());
        assert!(pass.is_ok());
        assert_eq!(pass.unwrap().as_ref(), "My-Secret-Passphrase42");
    }
}
