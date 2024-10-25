use anyhow::{anyhow, Context, Result};
use config::Config;
use serde::Deserialize;
use std::env::var;

pub struct Configuration {
    config: Config,
}

impl Configuration {
    pub fn new() -> Self {
        let home = var("XDG_CONFIG_HOME")
            .or(var("HOME"))
            .unwrap_or("".to_string());
        let system_config = format!("{}/.config/maconomy-cli/config", home);

        let config = Config::builder()
            // Config file `~/.config/maconomy-cli/config.toml`
            .add_source(config::File::with_name(&system_config).required(false))
            // Or `./config.toml`
            .add_source(config::File::with_name("config").required(false))
            // Add in settings from the environment (with a prefix of MACONOMY)
            // E.g. `MACONOMY_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(config::Environment::with_prefix("MACONOMY"))
            .build()
            .expect("Failed to read configuration");

        Self { config }
    }

    pub fn get_value<'a, T: Deserialize<'a>>(&self, value_name: &str) -> Result<T> {
        let error = format!(
            "Configuration value `{value_name}` is missing. Please set it in ./config.toml or \
                ~/.config/maconomy-cli/config.toml"
        );
        let value = self.config.get(value_name).context(error)?;
        Ok(value)
    }

    pub fn get_optional_value<'a, T: Deserialize<'a>>(
        &self,
        value_name: &str,
    ) -> Result<Option<T>> {
        let value = self.config.get(value_name);
        match value {
            Ok(value) => Ok(Some(value)),
            Err(config::ConfigError::NotFound(_)) => Ok(None),
            Err(err) => Err(anyhow!(
                "Failed to get optional value '{value_name}': {err}"
            )),
        }
    }
}
