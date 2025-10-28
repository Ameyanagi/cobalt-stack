//! Model Registry for LLM configuration management
//!
//! Loads and manages model definitions from models.toml with environment variable substitution.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelRegistryError {
    #[error("Failed to read models.toml: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse models.toml: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),

    #[error("Default model not found: {0}")]
    DefaultModelNotFound(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(default)]
    pub api_base: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub api_version: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub model_id: String,
    #[serde(default)]
    pub description: Option<String>,
    pub context_window: u32,
    pub max_output_tokens: u32,
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
    #[serde(default)]
    pub supports_function_calling: bool,
    pub cost_per_million_input_tokens: f64,
    pub cost_per_million_output_tokens: f64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub recommended_for: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelGroup {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub models: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModelsToml {
    default_provider: String,
    default_model: String,
    providers: HashMap<String, ProviderConfig>,
    models: Vec<ModelConfig>,
    #[serde(default)]
    model_groups: HashMap<String, ModelGroup>,
}

pub struct ModelRegistry {
    default_provider: String,
    default_model_id: String,
    providers: HashMap<String, ProviderConfig>,
    models: HashMap<String, ModelConfig>,
    model_groups: HashMap<String, ModelGroup>,
}

impl ModelRegistry {
    /// Load model registry from models.toml
    pub fn load() -> Result<Self, ModelRegistryError> {
        // Try loading from current directory first
        if let Ok(registry) = Self::load_from_path("models.toml") {
            return Ok(registry);
        }

        // Try loading from parent directory (for tests running in backend/)
        if let Ok(registry) = Self::load_from_path("../models.toml") {
            return Ok(registry);
        }

        Err(ModelRegistryError::FileReadError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "models.toml not found in current or parent directory",
        )))
    }

    /// Load model registry from a specific path (useful for testing)
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ModelRegistryError> {
        // Read the TOML file
        let content = fs::read_to_string(path)?;

        // Substitute environment variables
        let substituted = Self::substitute_env_vars(&content)?;

        // Parse TOML
        let toml_config: ModelsToml = toml::from_str(&substituted)?;

        // Build model lookup map
        let models: HashMap<String, ModelConfig> = toml_config
            .models
            .into_iter()
            .map(|m| (m.id.clone(), m))
            .collect();

        // Verify default model exists
        if !models.contains_key(&toml_config.default_model) {
            return Err(ModelRegistryError::DefaultModelNotFound(
                toml_config.default_model,
            ));
        }

        Ok(Self {
            default_provider: toml_config.default_provider,
            default_model_id: toml_config.default_model,
            providers: toml_config.providers,
            models,
            model_groups: toml_config.model_groups,
        })
    }

    /// Substitute ${VAR_NAME} patterns with environment variable values
    fn substitute_env_vars(content: &str) -> Result<String, ModelRegistryError> {
        let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
        let mut result = content.to_string();

        // Find all matches
        let matches: Vec<_> = re.captures_iter(content).collect();

        for cap in matches {
            let full_match = cap.get(0).unwrap().as_str();
            let var_name = cap.get(1).unwrap().as_str();

            // Get environment variable value
            let var_value = env::var(var_name)
                .map_err(|_| ModelRegistryError::EnvVarNotFound(var_name.to_string()))?;

            // Replace in result string
            result = result.replace(full_match, &var_value);
        }

        Ok(result)
    }

    /// Get a model by ID
    pub fn get_model(&self, id: &str) -> Result<&ModelConfig, ModelRegistryError> {
        self.models
            .get(id)
            .ok_or_else(|| ModelRegistryError::ModelNotFound(id.to_string()))
    }

    /// Get the default model
    pub fn default_model(&self) -> &ModelConfig {
        // Safe to unwrap because we verified in load()
        self.models.get(&self.default_model_id).unwrap()
    }

    /// Get default provider name
    pub fn default_provider(&self) -> &str {
        &self.default_provider
    }

    /// Get a provider configuration by name
    pub fn get_provider(&self, name: &str) -> Result<&ProviderConfig, ModelRegistryError> {
        self.providers
            .get(name)
            .ok_or_else(|| ModelRegistryError::ProviderNotFound(name.to_string()))
    }

    /// Get all models for a specific provider
    pub fn models_by_provider(&self, provider: &str) -> Vec<&ModelConfig> {
        self.models
            .values()
            .filter(|m| m.provider == provider && m.enabled)
            .collect()
    }

    /// Get all enabled models
    pub fn enabled_models(&self) -> Vec<&ModelConfig> {
        self.models
            .values()
            .filter(|m| m.enabled)
            .collect()
    }

    /// Get a model group by name
    pub fn get_model_group(&self, name: &str) -> Option<&ModelGroup> {
        self.model_groups.get(name)
    }

    /// Get all model groups
    pub fn model_groups(&self) -> &HashMap<String, ModelGroup> {
        &self.model_groups
    }

    /// Get all providers
    pub fn providers(&self) -> &HashMap<String, ProviderConfig> {
        &self.providers
    }

    /// Get all enabled providers
    pub fn enabled_providers(&self) -> Vec<(&String, &ProviderConfig)> {
        self.providers
            .iter()
            .filter(|(_, p)| p.enabled)
            .collect()
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_substitution() {
        // Set test environment variables
        env::set_var("TEST_VAR_1", "value1");
        env::set_var("TEST_VAR_2", "value2");

        let input = "api_base = \"${TEST_VAR_1}\" and key = \"${TEST_VAR_2}\"";
        let result = ModelRegistry::substitute_env_vars(input).unwrap();

        assert_eq!(result, "api_base = \"value1\" and key = \"value2\"");

        // Clean up
        env::remove_var("TEST_VAR_1");
        env::remove_var("TEST_VAR_2");
    }

    #[test]
    fn test_env_var_not_found() {
        let input = "api_base = \"${NONEXISTENT_VAR}\"";
        let result = ModelRegistry::substitute_env_vars(input);

        assert!(matches!(result, Err(ModelRegistryError::EnvVarNotFound(_))));
    }

    #[test]
    fn test_load_registry() {
        // This test requires actual models.toml and environment variables
        // Skip if not in proper environment
        if env::var("SAMBANOVA_API_BASE").is_err() {
            return;
        }

        let registry = ModelRegistry::load();
        assert!(registry.is_ok());

        if let Ok(registry) = registry {
            // Verify default model exists
            let default_model = registry.default_model();
            assert!(!default_model.id.is_empty());

            // Verify providers are loaded
            assert!(!registry.providers().is_empty());

            // Verify models are loaded
            assert!(!registry.enabled_models().is_empty());
        }
    }
}
