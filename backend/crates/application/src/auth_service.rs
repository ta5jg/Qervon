use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use qervon_domain::{Credential, CredentialRepository, User, UserId, UserRepository, UserRole};

use crate::ApplicationError;

pub struct AuthService<UR, CR>
where
    UR: UserRepository,
    CR: CredentialRepository,
{
    users: UR,
    credentials: CR,
}

impl<UR, CR> AuthService<UR, CR>
where
    UR: UserRepository,
    CR: CredentialRepository,
{
    pub fn new(users: UR, credentials: CR) -> Self {
        Self { users, credentials }
    }

    pub async fn register(
        &self,
        email: String,
        display_name: String,
        password: String,
        role: UserRole,
    ) -> Result<User, ApplicationError> {
        if password.len() < 12 {
            return Err(ApplicationError::Conflict(
                "password must contain at least 12 characters".into(),
            ));
        }
        if self.users.find_by_email(&email).await?.is_some() {
            return Err(ApplicationError::Conflict(
                "a user with this email already exists".into(),
            ));
        }
        let user = User::create(UserId::new(), email, display_name, role, Utc::now())?;
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| ApplicationError::Conflict("could not secure password".into()))?
            .to_string();
        self.users.create(&user).await?;
        self.credentials
            .save_credential(&Credential {
                user_id: user.id,
                password_hash,
                password_changed_at: Utc::now(),
            })
            .await?;
        Ok(user)
    }

    pub async fn authenticate(
        &self,
        email: &str,
        password: &str,
    ) -> Result<User, ApplicationError> {
        let user = self
            .users
            .find_by_email(email)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        if !user.is_active() {
            return Err(ApplicationError::Conflict("user is not active".into()));
        }
        let credential = self
            .credentials
            .find_credential(user.id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        let hash = PasswordHash::new(&credential.password_hash)
            .map_err(|_| ApplicationError::Conflict("invalid stored credential".into()))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .map_err(|_| ApplicationError::NotFound)?;
        Ok(user)
    }

    pub async fn revoke_refresh_session(&self, id: uuid::Uuid) -> Result<(), ApplicationError> {
        self.credentials.revoke_refresh_session(id).await?;
        Ok(())
    }

    pub fn now() -> chrono::DateTime<Utc> {
        Utc::now()
    }
}
