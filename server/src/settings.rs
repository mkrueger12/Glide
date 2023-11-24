use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct ServiceConfig {
    pub endpoint: String,
    pub models: Vec<String>
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub openai: ServiceConfig,
    pub cohere: ServiceConfig,
    pub anthropic: ServiceConfig,
}


impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
       // let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("server/src/config/config"))
            .build()?;

        // Now that we're done, let's access our configuration
        println!("provider: {:?}", s.get::<String>("database.url"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}