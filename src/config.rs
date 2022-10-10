use config::{Config, ConfigError, Environment, File};
use std::env;

const CONFIG_FILE_PREFIX: &str = "config/";

pub fn get_config() -> Config {
    let env = env::var("ENV").unwrap_or_else(|_| "dev".into());
    let config = Config::builder()
        // Start off by merging in the "default" configuration file
        .add_source(File::with_name(&format!("config/{}.toml", "default")))
        // Add in the current environment file
        .add_source(File::with_name(&format!("config/{}.toml", env)).required(true))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_KAFKA__HOST=example.com`
        .add_source(Environment::with_prefix("app").separator("__"))
        .build()
        // panic on error
        .unwrap();

    tracing::info!("Config:\n{:?}", config);
    config
}
