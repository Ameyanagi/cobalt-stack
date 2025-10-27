//! Token generation and hashing utilities for email verification.
//!
//! This module provides cryptographically secure token generation and hashing
//! functions for email verification workflows. Tokens are used in email verification
//! links and stored as SHA-256 hashes in the database.
//!
//! # Security
//!
//! - **Random Generation**: Uses cryptographically secure RNG via `rand::thread_rng()`
//! - **Hash Storage**: Tokens stored as SHA-256 hashes, never plaintext
//! - **Token Length**: 32 bytes (64 hex characters) for sufficient entropy
//! - **One-time Use**: Tokens expire after use or 24 hours
//!
//! # Examples
//!
//! ```
//! use cobalt_stack_backend::utils::token::{generate_verification_token, hash_token};
//!
//! // Generate a new verification token
//! let token = generate_verification_token();
//! assert_eq!(token.len(), 64); // 32 bytes as hex
//!
//! // Hash for database storage
//! let hash = hash_token(&token);
//! assert_eq!(hash.len(), 64); // SHA-256 hash as hex
//!
//! // Hashing is deterministic
//! assert_eq!(hash_token(&token), hash_token(&token));
//! ```

use rand::Rng;
use sha2::{Digest, Sha256};

/// Generate a cryptographically secure 32-byte random token as hex string.
///
/// Uses `rand::thread_rng()` to generate 32 random bytes and encodes them
/// as a 64-character hexadecimal string. Each generated token is statistically
/// unique with ~2^256 possible values.
///
/// # Returns
///
/// A 64-character hexadecimal string representing 32 random bytes.
///
/// # Examples
///
/// ```
/// use cobalt_stack_backend::utils::token::generate_verification_token;
///
/// let token = generate_verification_token();
/// assert_eq!(token.len(), 64);
/// assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
/// ```
///
/// # Security Notes
///
/// - Tokens are cryptographically random, suitable for security-sensitive operations
/// - Each token should be used only once and stored as a hash
/// - Tokens should expire after a reasonable time period (typically 24 hours)
#[must_use]
pub fn generate_verification_token() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    hex::encode(bytes)
}

/// Hash a token using SHA-256 for secure database storage.
///
/// Converts a plaintext token into a SHA-256 hash encoded as a 64-character
/// hexadecimal string. This allows storing tokens securely in the database
/// without exposing the actual token value.
///
/// # Arguments
///
/// * `token` - The plaintext token to hash
///
/// # Returns
///
/// A 64-character hexadecimal string representing the SHA-256 hash.
///
/// # Examples
///
/// ```
/// use cobalt_stack_backend::utils::token::hash_token;
///
/// let token = "my_secret_token";
/// let hash = hash_token(token);
///
/// assert_eq!(hash.len(), 64);
/// assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
///
/// // Hashing is deterministic
/// assert_eq!(hash_token(token), hash_token(token));
/// ```
///
/// # Security Notes
///
/// - SHA-256 is a one-way hash function (cannot be reversed)
/// - Same input always produces the same hash (deterministic)
/// - Database stores hashes, not plaintext tokens
/// - To verify a token, hash it and compare with stored hash
#[must_use]
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_verification_token_length() {
        let token = generate_verification_token();
        // Token should be 32 bytes = 64 hex characters
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_generate_verification_token_unique() {
        let token1 = generate_verification_token();
        let token2 = generate_verification_token();
        // Each token should be unique
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_verification_token_hex_chars() {
        let token = generate_verification_token();
        // Token should only contain valid hex characters
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_token_sha256_length() {
        let token = "abc123def456";
        let hashed = hash_token(token);
        // SHA-256 produces 64 hex characters (32 bytes)
        assert_eq!(hashed.len(), 64);
    }

    #[test]
    fn test_hash_token_deterministic() {
        let token = "test_token_12345";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);
        // Same input should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_different_inputs() {
        let hash1 = hash_token("token1");
        let hash2 = hash_token("token2");
        // Different inputs should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_hex_chars() {
        let token = "sample_token";
        let hashed = hash_token(token);
        // Hash should only contain valid hex characters
        assert!(hashed.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
