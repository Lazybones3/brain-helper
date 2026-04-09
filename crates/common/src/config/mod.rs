use std::sync::LazyLock;

use config::{Config, ConfigError, File};
use serde::Deserialize;
use datasource::DataSourceConfig;

mod datasource;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    datasource: DataSourceConfig
}

impl AppConfig {
    fn load() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(File::with_name("application"))
            .build()?;
        settings.try_deserialize()
    }

    pub fn datasource(&self) -> &DataSourceConfig {
        &self.datasource
    }
}

static GLOBAL_CONFIG: LazyLock<AppConfig> = LazyLock::new(|| AppConfig::load().expect("Failed to load Configuration"));

pub fn get() -> &'static AppConfig {
    &GLOBAL_CONFIG
}