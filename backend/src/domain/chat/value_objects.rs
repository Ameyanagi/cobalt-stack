//! Chat domain value objects
//!
//! Immutable value objects that represent domain concepts.

use serde::{Deserialize, Serialize};

/// Role of a message participant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Message from the user
    User,
    /// Message from the AI assistant
    Assistant,
    /// System message for behavior control
    System,
}

impl MessageRole {
    /// Convert role to string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
        }
    }

    /// Parse role from string
    ///
    /// # Errors
    ///
    /// Returns error if the string is not a valid role
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "system" => Ok(Self::System),
            _ => Err(format!("Invalid message role: {s}")),
        }
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_as_str() {
        assert_eq!(MessageRole::User.as_str(), "user");
        assert_eq!(MessageRole::Assistant.as_str(), "assistant");
        assert_eq!(MessageRole::System.as_str(), "system");
    }

    #[test]
    fn test_message_role_from_str() {
        assert_eq!(MessageRole::from_str("user").unwrap(), MessageRole::User);
        assert_eq!(
            MessageRole::from_str("assistant").unwrap(),
            MessageRole::Assistant
        );
        assert_eq!(MessageRole::from_str("system").unwrap(), MessageRole::System);
    }

    #[test]
    fn test_message_role_from_str_invalid() {
        assert!(MessageRole::from_str("invalid").is_err());
        assert!(MessageRole::from_str("").is_err());
    }

    #[test]
    fn test_message_role_display() {
        assert_eq!(format!("{}", MessageRole::User), "user");
        assert_eq!(format!("{}", MessageRole::Assistant), "assistant");
        assert_eq!(format!("{}", MessageRole::System), "system");
    }

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::User;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"user\"");

        let role = MessageRole::Assistant;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"assistant\"");
    }

    #[test]
    fn test_message_role_deserialization() {
        let role: MessageRole = serde_json::from_str("\"user\"").unwrap();
        assert_eq!(role, MessageRole::User);

        let role: MessageRole = serde_json::from_str("\"assistant\"").unwrap();
        assert_eq!(role, MessageRole::Assistant);
    }
}
