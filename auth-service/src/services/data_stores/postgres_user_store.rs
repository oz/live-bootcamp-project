use std::error::Error;

use argon2::{
    password_hash::{self, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
};

use sqlx::PgPool;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct PgUserRow {
    email: String,
    password_hash: String,
    requires_2fa: bool,
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password.as_ref().to_owned())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, requires_2fa)
            VALUES($1, $2, $3)
            "#,
            user.email.as_ref(),
            password_hash,
            user.requires_2fa,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let err = e.as_database_error().unwrap();
            if err.is_unique_violation() {
                UserStoreError::UserAlreadyExists
            } else {
                UserStoreError::UnexpectedError
            }
        })?;

        Ok(())
    }

    async fn get_user(&self, email: Email) -> Result<User, UserStoreError> {
        let row = sqlx::query_as!(
            PgUserRow,
            "SELECT email, password_hash, requires_2fa FROM users WHERE email = $1",
            email.as_ref(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UserStoreError::UserNotFound)?;

        let email = Email::parse(&row.email).expect("Invalid email in DB");
        let password = Password::parse(&row.password_hash).expect("Invalid password in DB");
        Ok(User::new(email, password, row.requires_2fa).clone())
    }

    async fn validate_user(&self, email: Email, password: Password) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        verify_password_hash(
            user.password.as_ref().to_owned(),
            password.as_ref().to_owned(),
        )
        .await
        .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(())
    }
}

// Helper function to verify if a given password matches an expected hash
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error>> {
    tokio::task::spawn_blocking(move || {
        let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;
        Argon2::default().verify_password(password_candidate.as_bytes(), &expected_password_hash)
    })
    .await
    // JoinError
    .map_err(|e| Box::<dyn Error>::from(e))?
    // PasswordHash error
    .map_err(|e| Box::<dyn Error>::from(e))?;

    Ok(())
}

// Helper function to hash passwords before persisting them in the database.
async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error>> {
    let handle = tokio::task::spawn_blocking(move || -> Result<String, password_hash::Error> {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        )
        .hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    });

    let password_hash = handle
        .await
        .map_err(|e| Box::<dyn Error>::from(e))?
        .map_err(|e| Box::<dyn Error>::from(e))?;

    Ok(password_hash.to_string())
}
