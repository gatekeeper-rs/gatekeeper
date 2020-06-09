use crate::{ConfigError, ConfigProvider, FileAwareConfigProvider};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// File aware in memory provider
#[derive(Default)]
pub struct InMemoryProvider {
    store: Arc<RwLock<HashMap<String, String>>>,
}

impl InMemoryProvider {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ConfigProvider for InMemoryProvider {
    fn get<T>(&self, key: &str) -> Result<T, ConfigError>
    where
        T: DeserializeOwned,
    {
        let read_guard = self.store.read().unwrap();
        let raw = read_guard.get(key).ok_or(ConfigError::NotFound)?;
        let deserialized =
            serde_json::from_str(&raw).map_err(|err| ConfigError::Other(Box::new(err)))?;

        Ok(deserialized)
    }

    fn has(&self, key: &str) -> Result<bool, ConfigError> {
        let read_guard = self.store.read().unwrap();

        Ok(read_guard.contains_key(key))
    }

    fn put<T>(&self, key: &str, value: T) -> Result<(), ConfigError>
    where
        T: DeserializeOwned + Serialize,
    {
        let serialized =
            serde_json::to_string(&value).map_err(|e| ConfigError::Other(Box::new(e)))?;
        let mut write_guard = self.store.write().unwrap();
        let _ = write_guard.insert(key.to_string(), serialized);

        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), ConfigError> {
        let mut write_guard = self.store.write().unwrap();
        let _ = write_guard.remove(key);

        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, ConfigError> {
        let read_guard = self.store.read().unwrap();

        Ok(read_guard.keys().cloned().collect())
    }
}

impl FileAwareConfigProvider for InMemoryProvider {
    fn load<P>(&self, path: P) -> Result<(), ConfigError>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).map_err(|err| ConfigError::Other(Box::new(err)))?;

        let mut write_guard = self.store.write().unwrap();

        let values: HashMap<String, String> =
            serde_json::from_reader(file).map_err(|err| ConfigError::Other(Box::new(err)))?;

        for (k, v) in values {
            write_guard.insert(k, v);
        }

        Ok(())
    }

    fn save<P>(&self, path: P) -> Result<(), ConfigError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        // try to create the directory
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| ConfigError::Other(Box::new(err)))?;
        }

        // create a temporary file to work with so we don't end up with a broken config
        let tmp_path = {
            let mut tmp_path = path.to_path_buf();
            tmp_path.set_extension("tmp");
            tmp_path
        };

        let mut tmp_file =
            File::create(&tmp_path).map_err(|err| ConfigError::Other(Box::new(err)))?;

        // acquire a read guard once the file is ready
        let read_guard = self.store.read().unwrap();

        // serialize the providers values and write it to the file
        serde_json::to_writer_pretty(&mut tmp_file, &*read_guard)
            .map_err(|err| ConfigError::Other(Box::new(err)))?;

        // move the temporary file to its final destination
        std::fs::rename(&tmp_path, path).map_err(|err| ConfigError::Other(Box::new(err)))?;

        Ok(())
    }
}
