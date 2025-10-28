use cobalt_stack_backend::infrastructure::llm::ModelRegistry;

fn main() {
    dotenvy::dotenv().ok();

    println!("Loading ModelRegistry from models.toml...");

    match ModelRegistry::load() {
        Ok(registry) => {
            println!("✓ ModelRegistry loaded successfully!\n");

            println!("=== Default Configuration ===");
            println!("Default provider: {}", registry.default_provider());
            let default_model = registry.default_model();
            println!("Default model: {} ({})", default_model.name, default_model.id);
            println!();

            println!("=== Enabled Providers ===");
            for (name, provider) in registry.enabled_providers() {
                println!("  • {} ({})", provider.name, name);
            }
            println!();

            println!("=== Enabled Models ===");
            for model in registry.enabled_models() {
                println!("  • {} ({}) - Provider: {}",
                    model.name,
                    model.id,
                    model.provider
                );
                println!("    Context: {}k, Max output: {}k, Streaming: {}",
                    model.context_window / 1024,
                    model.max_output_tokens / 1024,
                    model.supports_streaming
                );
            }
            println!();

            println!("=== Model Groups ===");
            for (name, group) in registry.model_groups() {
                println!("  • {} ({} models)", group.name, group.models.len());
            }
            println!();

            println!("=== SambaNova Models ===");
            for model in registry.models_by_provider("sambanova") {
                println!("  • {} ({})", model.name, model.id);
            }
            println!();

            println!("=== Azure Models ===");
            for model in registry.models_by_provider("azure") {
                println!("  • {} ({})", model.name, model.id);
            }

        }
        Err(e) => {
            eprintln!("✗ Failed to load ModelRegistry: {}", e);
            std::process::exit(1);
        }
    }
}
