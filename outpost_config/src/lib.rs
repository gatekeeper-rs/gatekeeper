use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::Path;
use thiserror::Error;

pub mod provider;

/// Key value config provider
///
/// The implementation may persist its values but is not forced to do so.
pub trait ConfigProvider {
    /// Get a specific value from the config and deserialize it to the given type.
    /// Returns a ConfigError::NotFound if the key doesn't or ConfigError::Other if something else
    /// goes wrong.
    fn get<T>(&self, key: &str) -> Result<T, ConfigError>
    where
        T: DeserializeOwned;

    /// Checks if the config contains the given key.
    /// Returns a ConfigError if the config could not be checked for some reason.
    fn has(&self, key: &str) -> Result<bool, ConfigError>;

    /// Insert a key value pair to the config
    fn put<T>(&self, key: &str, value: T) -> Result<(), ConfigError>
    where
        T: DeserializeOwned + Serialize;

    /// Deletes the given entry if it exists.
    /// Returns a ConfigError if the entry could not be deleted for some reason.
    fn delete(&self, key: &str) -> Result<(), ConfigError>;

    /// Lists all available keys in the config
    /// Returns a ConfigError if the keys could not be listed for some reason.
    fn list(&self) -> Result<Vec<String>, ConfigError>;
}

/// ConfigProvider that supports loading/saving its values from/to a file.
pub trait FileAwareConfigProvider: ConfigProvider {
    /// Load values from the given path.
    fn load<P>(&self, path: P) -> Result<(), ConfigError>
    where
        P: AsRef<Path>;

    /// Save the providers value to a file.
    fn save<P>(&self, path: P) -> Result<(), ConfigError>
    where
        P: AsRef<Path>;
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("not found")]
    NotFound,
    #[error("other: {0}")]
    Other(Box<dyn std::error::Error>),
}
