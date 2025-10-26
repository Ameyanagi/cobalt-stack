mod config;
mod handlers;
mod middleware;
mod models;
mod openapi;
mod services;
mod utils;

use axum::{http::{header, HeaderValue, Method}, middleware as axum_middleware, routing::{get, post}, Router};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use sea_orm::Database;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cobalt_stack_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Generate OpenAPI schema for frontend
    if let Err(e) = openapi::write_openapi_schema() {
        tracing::warn!("Failed to write OpenAPI schema: {}", e);
    } else {
        tracing::info!("OpenAPI schema generated at openapi/schema.json");
    }

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url).await.unwrap();
    tracing::info!("Database connected");

    // Initialize JWT config
    let jwt_config = services::auth::JwtConfig::from_env();

    // Create application state
    let state = handlers::auth::AppState {
        db: Arc::new(db),
        jwt_config: jwt_config.clone(),
    };

    // Build application router with state
    let app = create_app(state, jwt_config);

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn create_app(state: handlers::auth::AppState, jwt_config: services::auth::JwtConfig) -> Router {
    // Configure CORS with credentials support
    // Allow any origin ending with :2727 (frontend port) for development
    // In production, set specific allowed origins
    use tower_http::cors::AllowOrigin;

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _request_parts| {
            // Allow any origin that ends with :2727 (frontend port)
            // This enables access from localhost, LAN IPs, etc.
            origin.to_str()
                .map(|s| s.ends_with(":2727") || s == "http://localhost:2727" || s == "http://127.0.0.1:2727")
                .unwrap_or(false)
        }))
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::COOKIE,
        ])
        .allow_credentials(true);

    // Auth routes (public)
    let auth_public_routes = Router::new()
        .route("/api/auth/register", post(handlers::auth::register))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/refresh", post(handlers::auth::refresh_token))
        .with_state(state.clone());

    // Auth routes (protected)
    let auth_protected_routes = Router::new()
        .route("/api/auth/me", get(handlers::auth::get_current_user))
        .route("/api/auth/logout", post(handlers::auth::logout))
        .layer(axum_middleware::from_fn_with_state(
            jwt_config,
            middleware::auth::auth_middleware,
        ))
        .with_state(state);

    // Build main router
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .merge(auth_public_routes)
        .merge(auth_protected_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", openapi::ApiDoc::openapi()))
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

// TODO: Add integration tests later
