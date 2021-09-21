use anyhow::{Result, Context};

const APP_CONFIG_DIR: &'static str = "monzo-cli";
const APP_PREFIX: &'static str = "MONZO";

pub struct Config {
    data: config::Config
}

impl Config {
    pub fn init() -> Result<Self> {
        let config_path = dirs::config_dir()
            .ok_or_else(|| anyhow!("cannot locate config directory"))?
            .join(APP_CONFIG_DIR)
            .join("config.toml");

        let data = config::Config::new()
            .with_merged(config::File::from(config_path).required(false))?
            .with_merged(config::Environment::with_prefix(APP_PREFIX))?;

        Ok(Self { data })
    }

    pub fn set_token(&mut self, token: String) -> Result<&mut Self> {
        self.data
            .set("token", token)
            .context("could not set \"token\"")?;

        Ok(self)
    }

    pub fn get_token(&self) -> Result<String> {
        self.data
            .get_str("token")
            .context("\"token\" is not set")
    }
}
