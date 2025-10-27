# Backend Binaries

This directory contains standalone binaries for the Cobalt Stack backend.

## Available Binaries

### generate_openapi

Generates the OpenAPI schema without requiring a database connection.

**Usage:**
```bash
# Via Makefile (recommended)
make generate-openapi

# Or directly via cargo
cargo run --release --bin generate_openapi
```

**Output:** Creates `openapi/schema.json` in the project root.

**Purpose:** This standalone binary extracts the OpenAPI specification from the code annotations (`#[utoipa::path(...)]`) and writes it to a JSON file for:
- Frontend TypeScript type generation
- API documentation
- External tool integration

### seed_admin

Seeds the database with an initial admin user.

**Usage:**
```bash
# Via Makefile (recommended)
make seed-admin

# Or directly via cargo
cargo run --bin seed_admin
```

**Requirements:** Database must be running and `DATABASE_URL` must be set.

**Purpose:** Creates an initial admin user for testing and development.

## Adding New Binaries

To add a new binary:

1. Create a new file in `backend/src/bin/` (e.g., `my_binary.rs`)
2. Add a `main()` function
3. Run with: `cargo run --bin my_binary`

Binaries can import from the library crate using:
```rust
use cobalt_stack_backend::module_name;
```
