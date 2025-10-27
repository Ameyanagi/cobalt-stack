//! Chat feature configuration

use std::env;

use crate::application::chat::send_message::LlmConfig;

/// Chat feature configuration
#[derive(Debug, Clone)]
pub struct ChatConfig {
    /// Whether chat feature is enabled
    pub enabled: bool,
    /// LLM API configuration
    pub llm: LlmConfig,
    /// Maximum context messages to send to LLM
    pub max_context_messages: u64,
    /// Maximum message content length
    pub max_message_length: usize,
    /// Daily message quota per user
    pub daily_message_quota: u64,
    /// Rate limit (messages per minute)
    pub rate_limit_per_minute: u64,
}

impl ChatConfig {
    /// Load configuration from environment variables
    ///
    /// # Panics
    /// Panics if required environment variables are missing or invalid
    #[must_use]
    pub fn from_env() -> Self {
        let enabled = env::var("FEATURE_CHAT_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .expect("FEATURE_CHAT_ENABLED must be a boolean");

        let api_base = env::var("SAMBANOVA_API_BASE")
            .unwrap_or_else(|_| "https://api.sambanova.ai/v1".to_string());

        let api_key = env::var("SAMBANOVA_API_KEY")
            .expect("SAMBANOVA_API_KEY must be set when chat is enabled");

        let model = env::var("SAMBANOVA_MODEL")
            .unwrap_or_else(|_| "Meta-Llama-3.1-8B-Instruct".to_string());

        let max_context_messages = env::var("CHAT_MAX_CONTEXT_MESSAGES")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .expect("CHAT_MAX_CONTEXT_MESSAGES must be a number");

        let max_tokens = env::var("CHAT_MAX_TOKENS")
            .unwrap_or_else(|_| "2048".to_string())
            .parse()
            .expect("CHAT_MAX_TOKENS must be a number");

        let max_message_length = env::var("CHAT_MAX_MESSAGE_LENGTH")
            .unwrap_or_else(|_| "4000".to_string())
            .parse()
            .expect("CHAT_MAX_MESSAGE_LENGTH must be a number");

        let daily_message_quota = env::var("CHAT_DAILY_MESSAGE_QUOTA")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .expect("CHAT_DAILY_MESSAGE_QUOTA must be a number");

        let rate_limit_per_minute = env::var("CHAT_RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .expect("CHAT_RATE_LIMIT_PER_MINUTE must be a number");

        Self {
            enabled,
            llm: LlmConfig {
                api_base,
                api_key,
                model,
                max_context_messages,
                max_tokens,
            },
            max_context_messages,
            max_message_length,
            daily_message_quota,
            rate_limit_per_minute,
        }
    }
}
