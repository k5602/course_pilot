//! Keystore adapter using keyring crate.

use crate::domain::ports::{KeystoreError, SecretStore};

/// Native keystore adapter using OS credential stores.
pub struct NativeKeystore {
    service_name: String,
}

impl NativeKeystore {
    pub fn new(service_name: &str) -> Self {
        Self { service_name: service_name.to_string() }
    }
}

impl Default for NativeKeystore {
    fn default() -> Self {
        Self::new("course_pilot")
    }
}

impl SecretStore for NativeKeystore {
    fn store(&self, key: &str, secret: &str) -> Result<(), KeystoreError> {
        // TODO: Implement with keyring crate
        // keyring::Entry::new(&self.service_name, key)?.set_password(secret)?;
        let _ = (&self.service_name, key, secret);
        todo!("Implement with keyring crate")
    }

    fn retrieve(&self, key: &str) -> Result<Option<String>, KeystoreError> {
        // TODO: Implement with keyring crate
        let _ = (&self.service_name, key);
        todo!("Implement with keyring crate")
    }

    fn delete(&self, key: &str) -> Result<(), KeystoreError> {
        // TODO: Implement with keyring crate
        let _ = (&self.service_name, key);
        todo!("Implement with keyring crate")
    }

    fn exists(&self, key: &str) -> Result<bool, KeystoreError> {
        // TODO: Implement with keyring crate
        let _ = (&self.service_name, key);
        todo!("Implement with keyring crate")
    }
}
