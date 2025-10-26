// Email service for sending verification emails

use anyhow::Result;

/// Email sender trait for mock and SMTP implementations
pub trait EmailSender {
    fn send_verification_email(&self, to: &str, token: &str) -> Result<()>;
}

/// Mock email sender that logs to console
pub struct MockEmailSender;

impl EmailSender for MockEmailSender {
    fn send_verification_email(&self, to: &str, token: &str) -> Result<()> {
        tracing::info!(
            "📧 [MOCK EMAIL] Sending verification email to: {}",
            to
        );
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
