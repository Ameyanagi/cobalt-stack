use super::{AuthError, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash a password using Argon2id
///
/// Uses OWASP recommended parameters:
/// - Memory: 19 MiB (19456 KiB)
/// - Iterations: 2
/// - Parallelism: 1
pub fn hash_password(password: &str) -> Result<String> {
    // Validate password strength
    validate_password_strength(password)?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!("Password hashing failed: {:?}", e);
            AuthError::PasswordHashError
        })?;

    Ok(password_hash.to_string())
}

/// Verify a password against a hash using constant-time comparison
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|e| {
        tracing::error!("Failed to parse password hash: {:?}", e);
        AuthError::InvalidCredentials
    })?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Validate password meets security requirements
///
/// Requirements:
/// - Minimum 8 characters
/// - Maximum 128 characters (prevent DoS)
fn validate_password_strength(password: &str) -> Result<()> {
    let len = password.len();

    if len < 8 {
        return Err(AuthError::WeakPassword.into());
    }

    if len > 128 {
        return Err(AuthError::WeakPassword.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        // Verify hash format (PHC string format)
        assert!(hash.starts_with("$argon2"));
        assert!(hash.len() > 50); // Reasonable hash length
    }

    #[test]
    fn test_hash_password_weak_password_too_short() {
        let password = "short";
        let result = hash_password(password);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Weak password"));
    }

    #[test]
    fn test_hash_password_weak_password_too_long() {
        let password = "a".repeat(129);
        let result = hash_password(&password);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Weak password"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "correct_password_123";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "correct_password_123";
        let hash = hash_password(password).unwrap();

        let result = verify_password("wrong_password", &hash).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("any_password", "invalid_hash_format");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_produces_different_salts() {
        let password = "same_password_123";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_constant_time_comparison() {
        // This test doesn't prove constant-time, but verifies the function behavior
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        // Very different password
        let result1 = verify_password("completely_different", &hash).unwrap();
        assert!(!result1);

        // Slightly different password
        let result2 = verify_password("test_password_124", &hash).unwrap();
        assert!(!result2);

        // Both should return false regardless of difference magnitude
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_validate_password_strength_valid() {
        assert!(validate_password_strength("password123").is_ok());
        assert!(validate_password_strength("12345678").is_ok());
        assert!(validate_password_strength(&"a".repeat(128)).is_ok());
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        assert!(validate_password_strength("1234567").is_err());
        assert!(validate_password_strength("").is_err());
    }

    #[test]
    fn test_validate_password_strength_too_long() {
        assert!(validate_password_strength(&"a".repeat(129)).is_err());
    }
}
