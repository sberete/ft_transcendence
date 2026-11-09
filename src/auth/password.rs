// Thin wrapper around Argon2 for password hashing and verification, so
// the rest of the auth module never touches the crate's API directly.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

// Hashes a plaintext password with a fresh random salt.
pub fn hash_password(password: &str)
    -> Result<String, argon2::password_hash::Error>
{
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

// Checks a plaintext password against a previously stored Argon2 hash.
pub fn verify_password(password: &str, hash: &str)
    -> Result<bool, argon2::password_hash::Error>
{
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
