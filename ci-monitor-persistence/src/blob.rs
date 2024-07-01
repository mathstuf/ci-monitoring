// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use async_trait::async_trait;
use ci_monitor_core::data::{Blob, BlobReference};
use thiserror::Error;

pub mod filesystem;

/// Errors when interacting with blob persistence.
#[derive(Debug, Error)]
pub enum BlobPersistenceError {
    /// Authentication error.
    #[error("auth error: {}", details)]
    Auth {
        /// Error details.
        details: String,
    },
    /// Connection error.
    #[error("connection error: {}", details)]
    Connection {
        /// Error details.
        details: String,
    },
    /// Blob not found.
    #[error("blob not found")]
    NotFound,
    /// Other error.
    #[error("persistence error: {}", details)]
    Other {
        /// Error details.
        details: String,
    },
}

/// Blob verification error.
#[derive(Debug, Error)]
pub enum BlobPersistenceVerifyError {
    /// Returned blob had a reference mismatch.
    #[error("invalid blob; found: {}@{}", actual.algo().name(), actual.hash())]
    Invalid {
        /// The actual blob reference expected.
        actual: BlobReference,
    },
    /// Other error during verification.
    #[error("{}", source)]
    Inner {
        /// The inner error.
        #[from]
        source: BlobPersistenceError,
    },
}

impl BlobPersistenceVerifyError {
    fn invalid(actual: BlobReference) -> Self {
        Self::Invalid {
            actual,
        }
    }
}

/// A synchronous persistence store for blobs.
pub trait BlobPersistence {
    /// Persist a blob into storage.
    fn store(&self, blob: &Blob) -> Result<BlobReference, BlobPersistenceError>;
    /// Whether the storage contains a blob reference or not.
    fn contains(&self, blob: &BlobReference) -> Result<bool, BlobPersistenceError>;
    /// Fetch a blob from storage.
    fn fetch(&self, blob: &BlobReference) -> Result<Blob, BlobPersistenceError>;
    /// Verify a blob in the storage.
    fn verify(&self, blob: &BlobReference) -> Result<(), BlobPersistenceVerifyError> {
        let data = self.fetch(blob)?;
        let new_ref = BlobReference::for_blob(&data, blob.algo());
        if new_ref == *blob {
            Ok(())
        } else {
            Err(BlobPersistenceVerifyError::invalid(new_ref))
        }
    }
    /// Erase a blob from storage.
    fn erase(&self, blob: BlobReference) -> Result<(), BlobPersistenceError>;
}

/// An asynchronous persistence store for blobs.
#[async_trait]
pub trait BlobPersistenceAsync {
    /// Persist a blob into storage.
    async fn store(&self, blob: &Blob) -> Result<BlobReference, BlobPersistenceError>;
    /// Whether the storage contains a blob reference or not.
    async fn contains(&self, blob: &BlobReference) -> Result<bool, BlobPersistenceError>;
    /// Fetch a blob from storage.
    async fn fetch(&self, blob: &BlobReference) -> Result<Blob, BlobPersistenceError>;
    /// Verify a blob in the storage.
    async fn verify(&self, blob: &BlobReference) -> Result<(), BlobPersistenceVerifyError> {
        let data = self.fetch(blob).await?;
        let new_ref = BlobReference::for_blob(&data, blob.algo());
        if new_ref == *blob {
            Ok(())
        } else {
            Err(BlobPersistenceVerifyError::invalid(new_ref))
        }
    }
    /// Erase a blob from storage.
    async fn erase(&self, blob: BlobReference) -> Result<(), BlobPersistenceError>;
}
