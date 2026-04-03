use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub oidc: OidcConfig,
    pub keys: KeyConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
}

#[derive(Debug, Deserialize)]
pub struct OidcConfig {
    pub issuer_url: String,
}

#[derive(Debug, Deserialize)]
pub struct KeyConfig {
    pub key_path: PathBuf,
}

impl Config {
    /// Load configuration from `config.toml` in the current directory.
    pub fn load() -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string("config.toml")
            .map_err(|e| ConfigError::FileNotFound {
                path: "config.toml".to_string(),
                source: e,
            })?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError {
                source: e,
            })?;

        config.validate()?;

        Ok(config)
    }

    /// Validate semantic constraints that the type system can't enforce.
    fn validate(&self) -> Result<(), ConfigError> {
        if self.oidc.issuer_url.ends_with('/') {
            return Err(ConfigError::InvalidValue {
                field: "oidc.issuer_url".to_string(),
                reason: "issuer_url must not end with a trailing slash".to_string(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Config file not found at '{path}': {source}")]
    FileNotFound {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file: {source}")]
    ParseError {
        #[source]
        source: toml::de::Error,
    },

    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue { field: String, reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config_parses() {
        let toml_str = r#"
            [server]
            bind_address = "127.0.0.1:3000"

            [oidc]
            issuer_url = "http://localhost:3000"

            [keys]
            key_path = "./keys"
        "#;

        let config: Config = toml::from_str(toml_str).expect("should parse valid config");
        assert_eq!(config.server.bind_address, "127.0.0.1:3000");
        assert_eq!(config.oidc.issuer_url, "http://localhost:3000");
    }

    #[test]
    fn test_trailing_slash_rejected() {
        let toml_str = r#"
            [server]
            bind_address = "127.0.0.1:3000"

            [oidc]
            issuer_url = "http://localhost:3000/"

            [keys]
            key_path = "./keys"
        "#;

        let config: Config = toml::from_str(toml_str).expect("should parse");
        let result = config.validate();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.to_string().contains("trailing slash"));
    }
}