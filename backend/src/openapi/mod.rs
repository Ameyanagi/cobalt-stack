//! OpenAPI specification and documentation generation.
//!
//! This module defines the OpenAPI 3.0 specification for the entire API
//! using the `utoipa` crate. The spec is used for Swagger UI documentation
//! and can be exported for frontend TypeScript type generation.
//!
//! # Components
//!
//! - **Paths**: All API endpoints from handlers
//! - **Schemas**: Request/response models and enums
//! - **Security**: Bearer token authentication scheme
//! - **Tags**: Endpoint categorization
//!
//! # Swagger UI
//!
//! Interactive API documentation available at:
//! ```text
//! http://localhost:3000/swagger-ui
//! ```
//!
//! # Frontend Integration
//!
//! OpenAPI schema is written to `openapi/schema.json` at startup for
//! frontend type generation:
//!
//! ```bash
//! # Frontend can generate types with:
//! npx openapi-typescript ./backend/openapi/schema.json -o ./types/api.ts
//! ```
//!
//! # Adding New Endpoints
//!
//! To document a new endpoint:
//!
//! 1. Add `#[utoipa::path(...)]` attribute to handler function
//! 2. Add handler path to `paths(...)` in [`ApiDoc`]
//! 3. Add request/response types to `schemas(...)` if needed
//! 4. Restart server to regenerate schema
//!
//! # Examples
//!
//! ```no_run
//! use cobalt_stack_backend::openapi::ApiDoc;
//! use utoipa::OpenApi;
//!
//! // Get OpenAPI spec as JSON
//! let spec = ApiDoc::openapi();
//! let json = serde_json::to_string_pretty(&spec).unwrap();
//! println!("{}", json);
//! ```

use utoipa::OpenApi;

/// OpenAPI 3.0 specification for the Cobalt Stack API.
///
/// This struct defines the complete API documentation including all endpoints,
/// request/response schemas, security schemes, and metadata. The specification
/// is automatically generated from handler attributes and model derives.
///
/// # Accessing the Spec
///
/// - **Swagger UI**: http://localhost:3000/swagger-ui
/// - **JSON Spec**: http://localhost:3000/openapi.json
/// - **File Export**: `openapi/schema.json` (generated at startup)
///
/// # Sections
///
/// - **Health**: Health check endpoints
/// - **Authentication**: User auth and email verification
/// - **Admin**: Admin user management endpoints
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::health_check,
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::refresh_token,
        crate::handlers::auth::logout,
        crate::handlers::auth::get_current_user,
        crate::handlers::auth::send_verification_email,
        crate::handlers::auth::verify_email,
        crate::handlers::admin::list_users,
        crate::handlers::admin::get_user,
        crate::handlers::admin::disable_user,
        crate::handlers::admin::enable_user,
        crate::handlers::admin::get_stats,
    ),
    components(
        schemas(
            crate::handlers::health::HealthResponse,
            crate::handlers::auth::RegisterRequest,
            crate::handlers::auth::LoginRequest,
            crate::handlers::auth::AuthResponse,
            crate::handlers::auth::UserResponse,
            crate::handlers::auth::ErrorResponse,
            crate::handlers::auth::VerifyEmailRequest,
            crate::handlers::auth::MessageResponse,
            crate::handlers::admin::AdminUserResponse,
            crate::handlers::admin::UserListResponse,
            crate::handlers::admin::AdminStatsResponse,
            crate::handlers::admin::MessageResponse,
            crate::models::sea_orm_active_enums::UserRole,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "Authentication", description = "User authentication and email verification"),
        (name = "Admin", description = "Admin user management endpoints")
    ),
    info(
        title = "Cobalt Stack API",
        version = "0.1.0",
        description = "Full-stack application with Rust backend (Axum + SeaORM) and Next.js frontend",
        contact(
            name = "API Support"
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

use utoipa::Modify;

/// Security scheme modifier that adds Bearer token authentication.
///
/// This struct implements the `Modify` trait to add JWT Bearer authentication
/// to the OpenAPI specification. The security scheme is referenced by protected
/// endpoints using the `security(("bearer_auth" = []))` attribute.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            )
        }
    }
}

/// Write OpenAPI schema to file for frontend type generation.
///
/// Generates the OpenAPI specification as JSON and writes it to
/// `openapi/schema.json`. This file can be used by frontend tools
/// like `openapi-typescript` to generate TypeScript types.
///
/// # Returns
///
/// - `Ok(())` - Schema successfully written
/// - `Err(_)` - File I/O error
///
/// # Examples
///
/// ```no_run
/// use cobalt_stack_backend::openapi::write_openapi_schema;
///
/// // Called at application startup
/// write_openapi_schema().expect("Failed to write OpenAPI schema");
/// println!("OpenAPI schema written to openapi/schema.json");
/// ```
///
/// # Frontend Usage
///
/// ```bash
/// # Generate TypeScript types from schema
/// npx openapi-typescript ./backend/openapi/schema.json -o ./types/api.ts
///
/// # Use generated types in frontend
/// import type { paths } from './types/api';
/// type LoginRequest = paths['/api/auth/login']['post']['requestBody']['content']['application/json'];
/// ```
pub fn write_openapi_schema() -> Result<(), std::io::Error> {
    let doc = ApiDoc::openapi();
    let yaml = serde_json::to_string_pretty(&doc).unwrap();

    // Create openapi directory if it doesn't exist
    std::fs::create_dir_all("openapi")?;

    // Write schema as JSON (easier for openapi-typescript to parse)
    std::fs::write("openapi/schema.json", yaml)?;

    Ok(())
}
