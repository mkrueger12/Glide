
use std::net::Ipv4Addr;

use config::{Config, ConfigError, File};
use serde::Deserialize;
use crate::config::settings;
use lazy_static::lazy_static;

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub endpoint: String,
    pub models: Vec<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct GenericServiceConfig {
    pub ip: Ipv4Addr,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub openai: ServiceConfig,
    pub cohere: ServiceConfig,
    pub generic: GenericServiceConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("src/config/config.toml"))
            .build()?;

        // Now that we're done, let's access our configuration
        //println!("provider: {:?}", s.get::<String>("database.url"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}

lazy_static! {
    pub static ref CONF: Result<settings::Settings, ConfigError> = {
        settings::Settings::new()
    };
}
