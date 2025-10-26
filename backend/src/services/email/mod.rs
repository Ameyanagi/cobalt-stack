//! Email delivery service for user communication.
//!
//! This module provides email sending functionality with support for both
//! mock (development) and production SMTP implementations. Currently handles
//! email verification but designed to be extensible for other email types.
//!
//! # Architecture
//!
//! - **EmailSender trait**: Abstraction for different email backends
//! - **MockEmailSender**: Development implementation that logs to console
//! - **verification**: Email verification token management
//!
//! # Usage
//!
//! ```no_run
//! use cobalt_stack_backend::services::email::{EmailSender, MockEmailSender};
//!
//! let sender = MockEmailSender;
//! sender.send_verification_email("user@example.com", "abc123token").unwrap();
//! ```
//!
//! # Future Extensions
//!
//! - SMTP implementation for production email delivery
//! - Template-based email rendering
//! - Password reset emails
//! - Welcome emails
//! - Notification emails

mod verification;

use anyhow::Result;
pub use verification::{create_verification_token, verify_email_token};

/// Abstraction for email sending implementations.
///
/// This trait allows swapping between mock (development) and real (production)
/// email backends without changing handler code. Implementations handle the
/// actual delivery mechanism.
///
/// # Implementations
///
/// - [`MockEmailSender`]: Logs to console instead of sending real emails
/// - Future: `SmtpEmailSender` for production SMTP delivery
///
/// # Examples
///
/// ```
/// use cobalt_stack_backend::services::email::{EmailSender, MockEmailSender};
///
/// fn send_verification(sender: &impl EmailSender, email: &str, token: &str) {
///     sender.send_verification_email(email, token).unwrap();
/// }
///
/// let mock_sender = MockEmailSender;
/// send_verification(&mock_sender, "user@example.com", "abc123");
/// ```
pub trait EmailSender {
    /// Send an email verification link to the user.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient email address
    /// * `token` - Verification token to include in link
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Email sent successfully (or logged for mock)
    /// - `Err(_)` - Email delivery failed
    fn send_verification_email(&self, to: &str, token: &str) -> Result<()>;
}

/// Mock email sender for development and testing.
///
/// Instead of sending real emails, this implementation logs verification
/// links to the console. Useful for local development and automated testing.
///
/// # Examples
///
/// ```
/// use cobalt_stack_backend::services::email::{EmailSender, MockEmailSender};
///
/// let sender = MockEmailSender;
/// sender.send_verification_email("test@example.com", "abc123").unwrap();
/// // Logs: "📧 [MOCK EMAIL] Sending verification email to: test@example.com"
/// // Logs: "📧 [MOCK EMAIL] Verification link: http://localhost:2727/verify-email?token=abc123"
/// ```
pub struct MockEmailSender;

impl EmailSender for MockEmailSender {
    fn send_verification_email(&self, to: &str, token: &str) -> Result<()> {
        tracing::info!("📧 [MOCK EMAIL] Sending verification email to: {}", to);
        tracing::info!(
            "📧 [MOCK EMAIL] Verification link: http://localhost:2727/verify-email?token={}",
            token
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_email_sender_returns_ok() {
        let sender = MockEmailSender;
        let result = sender.send_verification_email("test@example.com", "abc123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_email_sender_handles_empty_email() {
        let sender = MockEmailSender;
        let result = sender.send_verification_email("", "abc123");
        // Mock sender should still work even with invalid input
        // (validation happens at API layer)
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_email_sender_handles_long_token() {
        let sender = MockEmailSender;
        let long_token = "a".repeat(1000);
        let result = sender.send_verification_email("test@example.com", &long_token);
        assert!(result.is_ok());
    }
}
