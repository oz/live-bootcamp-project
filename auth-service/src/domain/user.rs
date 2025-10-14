//
// The User struct should contain 3 fields. email, which is a String;
// password, which is also a String; and requires_2fa, which is a boolean.
#[derive(Debug)]
pub struct User {
    pub email: String,
    pub password: String,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: &str, password: &str, requires_2fa: bool) -> Self {
        Self {
            email: String::from(email),
            password: String::from(password),
            requires_2fa,
        }
    }
}
