// Token generation and hashing utilities for email verification

use rand::Rng;
use sha2::{Digest, Sha256};

/// Generate a cryptographically secure 32-byte random token as hex string
pub fn generate_verification_token() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    hex::encode(bytes)
}

/// Hash a token using SHA-256 for storage
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
