use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::health_check,
    ),
    components(
        schemas(crate::handlers::health::HealthResponse)
    ),
    tags(
        (name = "health", description = "Health check endpoints")
    ),
    info(
        title = "Cobalt Stack API",
        version = "0.1.0",
        description = "Full-stack application with Rust backend (Axum + SeaORM) and Next.js frontend",
        contact(
            name = "API Support"
        )
    )
)]
pub struct ApiDoc;

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
