//! Secret storage port for API keys.

/// Error type for keystore operations.
#[derive(Debug, thiserror::Error)]
pub enum KeystoreError {
    #[error("Storage error: {0}")]
    Storage(String),
}

/// Port for secure credential storage.
pub trait SecretStore: Send + Sync {
    /// Stores a secret.
    fn store(&self, key: &str, secret: &str) -> Result<(), KeystoreError>;

    /// Retrieves a secret.
    fn retrieve(&self, key: &str) -> Result<Option<String>, KeystoreError>;

    /// Deletes a secret.
    fn delete(&self, key: &str) -> Result<(), KeystoreError>;

    /// Checks if a secret exists.
    fn exists(&self, key: &str) -> Result<bool, KeystoreError>;
}
