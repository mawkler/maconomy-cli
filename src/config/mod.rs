use anyhow::{anyhow, Context, Result};
use config::Config;
use serde::Deserialize;

const DEFAULT_PATH: &str = "~/.config/maconomy-cli/config";

#[derive(Debug)]
pub struct Configuration(Config);

impl Configuration {
    pub fn new(config_str: Option<String>) -> Self {
        let builder = Config::builder()
            // Config file `~/.config/maconomy-cli/config.toml`
            .add_source(config::File::with_name(DEFAULT_PATH).required(false))
            // Or `./config.toml`
            .add_source(config::File::with_name("config").required(false))
            // Add in settings from environment variables (with a prefix of `MACONOMY__`)
            //
            // E.g. `MACONOMY__AUTHENTICATION__SSO__COOKIE_PATH=foo/bar/cookie ./target/maconomy`
            // runs with `authentication.sso.cookie_path` set to `foo/bar/cookie` (notice how the
            // `.`s are replaced with double underscores)
            .add_source(config::Environment::with_prefix("MACONOMY").separator("__"));

        let builder = match config_str {
            Some(config_str) => {
                // Literal config passed in through stdin
                let source = config::File::from_str(&config_str, config::FileFormat::Toml);
                builder.add_source(source)
            }
            None => builder,
        };

        Self(builder.build().expect("Failed to read configuration"))
    }

    pub fn get_value<'a, T: Deserialize<'a>>(&self, value_name: &str) -> Result<T> {
        let error = format!(
            "Configuration value `{value_name}` is missing. Please set it in ./config.toml or \
                ~/.config/maconomy-cli/config.toml"
        );
        let value = self.0.get(value_name).context(error)?;
        Ok(value)
    }

    pub fn get_optional_value<'a, T: Deserialize<'a>>(
        &self,
        value_name: &str,
    ) -> Result<Option<T>> {
        let value = self.0.get(value_name);
        match value {
            Ok(value) => Ok(Some(value)),
            Err(config::ConfigError::NotFound(_)) => Ok(None),
            Err(err) => Err(anyhow!(
                "Failed to get optional value '{value_name}': {err}"
            )),
        }
    }
}
