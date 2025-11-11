use std::error::Error;
use tracing;

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
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
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
            let is_unique = e
                .as_database_error()
                .filter(|db_err| db_err.is_unique_violation());
            match is_unique {
                Some(_) => UserStoreError::UserAlreadyExists,
                _ => UserStoreError::UnexpectedError,
            }
        })?;

        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
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

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
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

#[tracing::instrument(name = "Verify password hash", skip_all)]
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
    .map_err(Box::<dyn Error>::from)?
    // PasswordHash error
    .map_err(Box::<dyn Error>::from)?;

    Ok(())
}

#[tracing::instrument(name = "Computing password hash", skip_all)]
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
        .map_err(Box::<dyn Error>::from)?
        .map_err(Box::<dyn Error>::from)?;

    Ok(password_hash.to_string())
}
