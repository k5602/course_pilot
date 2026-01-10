//! Keystore adapter using native OS credential storage.

use keyring::Entry;

use crate::domain::ports::{KeystoreError, SecretStore};

const SERVICE_NAME: &str = "course_pilot";

/// Native keystore adapter using OS credential stores.
pub struct NativeKeystore;

impl NativeKeystore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativeKeystore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for NativeKeystore {
    fn store(&self, key: &str, secret: &str) -> Result<(), KeystoreError> {
        let entry =
            Entry::new(SERVICE_NAME, key).map_err(|e| KeystoreError::Storage(e.to_string()))?;

        entry.set_password(secret).map_err(|e| KeystoreError::Storage(e.to_string()))
    }

    fn retrieve(&self, key: &str) -> Result<Option<String>, KeystoreError> {
        let entry =
            Entry::new(SERVICE_NAME, key).map_err(|e| KeystoreError::Storage(e.to_string()))?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(KeystoreError::Storage(e.to_string())),
        }
    }

    fn delete(&self, key: &str) -> Result<(), KeystoreError> {
        let entry =
            Entry::new(SERVICE_NAME, key).map_err(|e| KeystoreError::Storage(e.to_string()))?;

        match entry.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
            Err(e) => Err(KeystoreError::Storage(e.to_string())),
        }
    }

    fn exists(&self, key: &str) -> Result<bool, KeystoreError> {
        Ok(self.retrieve(key)?.is_some())
    }
}
