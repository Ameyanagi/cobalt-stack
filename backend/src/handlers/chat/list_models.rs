//! List available LLM models endpoint

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::handlers::chat::ChatState;

/// Model information for API response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub context_window: u32,
    pub max_output_tokens: u32,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub cost_per_million_input_tokens: f64,
    pub cost_per_million_output_tokens: f64,
    pub tags: Vec<String>,
    pub recommended_for: Vec<String>,
}

/// Model group information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ModelGroupInfo {
    pub name: String,
    pub description: Option<String>,
    pub models: Vec<String>,
}

/// API response with models and groups
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListModelsResponse {
    pub models: Vec<ModelInfo>,
    pub groups: Vec<ModelGroupInfo>,
    pub default_model: String,
}

/// Get list of available LLM models
///
/// Returns all enabled models from the model registry along with their metadata.
///
/// # Errors
/// Returns HTTP error if:
/// - Model registry cannot be accessed (500)
#[utoipa::path(
    get,
    path = "/api/v1/chat/models",
    tag = "chat",
    responses(
        (status = 200, description = "List of available models", body = ListModelsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_models(
    State(state): State<ChatState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let registry = state.provider_factory.model_registry();

    // Get all enabled models
    let enabled_models = registry.enabled_models();

    let models: Vec<ModelInfo> = enabled_models
        .into_iter()
        .map(|model| ModelInfo {
            id: model.id.clone(),
            name: model.name.clone(),
            provider: model.provider.clone(),
            description: model.description.clone(),
            context_window: model.context_window,
            max_output_tokens: model.max_output_tokens,
            supports_streaming: model.supports_streaming,
            supports_function_calling: model.supports_function_calling,
            cost_per_million_input_tokens: model.cost_per_million_input_tokens,
            cost_per_million_output_tokens: model.cost_per_million_output_tokens,
            tags: model.tags.clone(),
            recommended_for: model.recommended_for.clone(),
        })
        .collect();

    // Get model groups
    let groups: Vec<ModelGroupInfo> = registry
        .model_groups()
        .iter()
        .map(|(_, group)| ModelGroupInfo {
            name: group.name.clone(),
            description: group.description.clone(),
            models: group.models.clone(),
        })
        .collect();

    let default_model = registry.default_model().id.clone();

    Ok(Json(ListModelsResponse {
        models,
        groups,
        default_model,
    }))
}
