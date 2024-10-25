use anyhow::{anyhow, Context, Result};
use config::Config;
use serde::Deserialize;

pub struct Configuration {
    config: Config,
}

impl Configuration {
    pub fn new() -> Self {
        let config_path = shellexpand::tilde("~/.config/maconomy-cli/config").to_string();
        let config = Config::builder()
            // Config file `~/.config/maconomy-cli/config.toml`
            .add_source(config::File::with_name(&config_path).required(false))
            // Or `./config.toml`
            .add_source(config::File::with_name("config").required(false))
            // Add in settings from environment variables (with a prefix of `MACONOMY__`)
            //
            // E.g. `MACONOMY__AUTHENTICATION__SSO__COOKIE_PATH=foo/bar/cookie ./target/maconomy`
            // runs with `authentication.sso.cookie_path` set to `foo/bar/cookie` (notice how the
            // `.`s are replaced with double underscores)
            .add_source(config::Environment::with_prefix("MACONOMY").separator("__"))
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
