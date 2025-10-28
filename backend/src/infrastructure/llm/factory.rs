//! Provider factory for routing to appropriate LLM provider
//!
//! Creates and manages LLM provider instances based on model registry configuration.

use super::{
    model_registry::ModelRegistry,
    provider::{LlmProvider, LlmProviderError, LlmResult},
    sambanova_provider::SambaNovaProvider,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Factory for creating and managing LLM providers
pub struct ProviderFactory {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    model_registry: ModelRegistry,
}

impl ProviderFactory {
    /// Create a new provider factory
    ///
    /// # Errors
    /// Returns error if model registry cannot be loaded or providers cannot be initialized
    pub fn new() -> LlmResult<Self> {
        // Load model registry
        let model_registry =
            ModelRegistry::load().map_err(|e| LlmProviderError::ConfigError(e.to_string()))?;

        let mut providers: HashMap<String, Arc<dyn LlmProvider>> = HashMap::new();

        // Initialize SambaNova provider if configured
        if let Ok(provider_config) = model_registry.get_provider("sambanova") {
            if provider_config.enabled {
                let api_base = provider_config
                    .api_base
                    .clone()
                    .ok_or_else(|| LlmProviderError::ConfigError("SambaNova api_base missing".to_string()))?;
                let api_key = provider_config
                    .api_key
                    .clone()
                    .ok_or_else(|| LlmProviderError::ConfigError("SambaNova api_key missing".to_string()))?;

                let provider = SambaNovaProvider::new(api_base, api_key, model_registry.clone());
                providers.insert("sambanova".to_string(), Arc::new(provider));
                tracing::info!("Initialized SambaNova provider");
            }
        }

        // TODO: Initialize Azure AI provider when implemented
        // if let Ok(provider_config) = model_registry.get_provider("azure") {
        //     if provider_config.enabled {
        //         let provider = AzureAIProvider::new(...);
        //         providers.insert("azure".to_string(), Arc::new(provider));
        //         tracing::info!("Initialized Azure AI provider");
        //     }
        // }

        if providers.is_empty() {
            return Err(LlmProviderError::ConfigError(
                "No LLM providers configured".to_string(),
            ));
        }

        Ok(Self {
            providers,
            model_registry,
        })
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> LlmResult<Arc<dyn LlmProvider>> {
        self.providers
            .get(name)
            .cloned()
            .ok_or_else(|| LlmProviderError::ConfigError(format!("Provider '{}' not found", name)))
    }

    /// Get the provider for a specific model ID
    pub fn get_provider_for_model(&self, model_id: &str) -> LlmResult<Arc<dyn LlmProvider>> {
        // Look up model in registry
        let model = self
            .model_registry
            .get_model(model_id)
            .map_err(|e| LlmProviderError::ConfigError(e.to_string()))?;

        // Get provider for this model
        self.get_provider(&model.provider)
    }

    /// Get the default provider
    pub fn default_provider(&self) -> LlmResult<Arc<dyn LlmProvider>> {
        let provider_name = self.model_registry.default_provider();
        self.get_provider(provider_name)
    }

    /// Get the model registry
    pub fn model_registry(&self) -> &ModelRegistry {
        &self.model_registry
    }

    /// List all available provider names
    pub fn available_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

// Implement Clone for ModelRegistry to support provider factory
impl Clone for ModelRegistry {
    fn clone(&self) -> Self {
        // Load a fresh instance from file
        // This is safe because ModelRegistry is immutable after loading
        Self::load().expect("Failed to reload ModelRegistry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        // Skip if models.toml not available
        let Ok(factory) = ProviderFactory::new() else {
            eprintln!("Skipping test: models.toml not found");
            return;
        };

        assert!(!factory.available_providers().is_empty());
    }

    #[test]
    fn test_get_provider_for_model() {
        // Skip if models.toml not available
        let Ok(factory) = ProviderFactory::new() else {
            eprintln!("Skipping test: models.toml not found");
            return;
        };

        // Test with SambaNova model
        let provider = factory.get_provider_for_model("llama-3.3-70b");
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "SambaNova");
    }

    #[test]
    fn test_default_provider() {
        // Skip if models.toml not available
        let Ok(factory) = ProviderFactory::new() else {
            eprintln!("Skipping test: models.toml not found");
            return;
        };

        let provider = factory.default_provider();
        assert!(provider.is_ok());
    }

    #[test]
    fn test_get_provider_by_name() {
        // Skip if models.toml not available
        let Ok(factory) = ProviderFactory::new() else {
            eprintln!("Skipping test: models.toml not found");
            return;
        };

        let provider = factory.get_provider("sambanova");
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "SambaNova");

        let provider = factory.get_provider("nonexistent");
        assert!(provider.is_err());
    }
}
