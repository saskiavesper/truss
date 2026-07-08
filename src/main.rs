mod config;
mod validation;

use config::must_load;

fn main() {
    // Load configuration from environment variables
    // Set TRUSS_ENVIRONMENT=test|dev|local|prod
    let cfg = must_load();

    println!("Starting application in {} environment", cfg.environment);

    // Example: Use environment checks
    if cfg.environment.is_production() {
        println!("Running in production mode - enabling optimizations");
    } else if cfg.environment.is_development() {
        println!("Running in development mode - enabling debug features");
    } else if cfg.environment.is_test() {
        println!("Running in test mode - using mock services");
    } else if cfg.environment.is_local() {
        println!("Running in local mode - using local services");
    }

    // Your application logic here...
}
