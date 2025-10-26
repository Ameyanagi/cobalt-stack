use utoipa::OpenApi;

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
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "Authentication", description = "User authentication and email verification")
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

/// Write OpenAPI schema to file for frontend type generation
pub fn write_openapi_schema() -> Result<(), std::io::Error> {
    let doc = ApiDoc::openapi();
    let yaml = serde_json::to_string_pretty(&doc).unwrap();

    // Create openapi directory if it doesn't exist
    std::fs::create_dir_all("openapi")?;

    // Write schema as JSON (easier for openapi-typescript to parse)
    std::fs::write("openapi/schema.json", yaml)?;

    Ok(())
}
