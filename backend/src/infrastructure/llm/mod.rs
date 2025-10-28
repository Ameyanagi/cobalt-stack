//! LLM Infrastructure
//!
//! Contains model registry and provider implementations for LLM services.

pub mod model_registry;

pub use model_registry::{ModelConfig, ModelRegistry, ProviderConfig};
