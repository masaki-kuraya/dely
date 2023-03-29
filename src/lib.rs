use config::{Config, ConfigError};
use serde::Deserialize;

pub mod domain;
pub mod infrastructure;

#[derive(Clone, Debug, Deserialize)]
pub struct DelyConfig {
    pub eventstore: EventStore,
    pub meilisearch: MeiliSearch,
    pub logger: Logger,
}

impl DelyConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(config::File::with_name("dely.toml"))
            .add_source(config::Environment::with_prefix("DELY").separator("_"))
            .build()?
            .try_deserialize::<DelyConfig>()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct EventStore {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MeiliSearch {
    pub url: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Logger {
    pub level: Level,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Level {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl From<&Level> for tracing::Level {
    fn from(value: &Level) -> Self {
        match value {
            Level::TRACE => tracing::Level::TRACE,
            Level::DEBUG => tracing::Level::DEBUG,
            Level::INFO => tracing::Level::INFO,
            Level::WARN => tracing::Level::WARN,
            Level::ERROR => tracing::Level::ERROR,
        }
    }
}
