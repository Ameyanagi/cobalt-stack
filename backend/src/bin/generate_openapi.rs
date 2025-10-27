//! OpenAPI Schema Generator
//!
//! Standalone binary to generate OpenAPI schema without database connection.
//! This binary only generates the schema file and exits immediately.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin generate_openapi
//! ```
//!
//! # Output
//!
//! Writes OpenAPI schema to `openapi/schema.json` in project root.

use cobalt_stack_backend::openapi;

fn main() {
    // Generate OpenAPI schema
    match openapi::write_openapi_schema() {
        Ok(_) => {
            println!("✅ OpenAPI schema generated at openapi/schema.json");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Failed to generate OpenAPI schema: {}", e);
            std::process::exit(1);
        }
    }
}
