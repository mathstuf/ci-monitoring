// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use digest::Digest;

/// Content hash used to compute uniqueness for a blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentHash {
    /// SHA-256 hashing algorithm.
    Sha256,
    /// SHA-512 hashing algorithm.
    Sha512,
}

impl ContentHash {
    /// Hash a blob.
    fn hash_blob(self, data: &[u8]) -> String {
        match self {
            Self::Sha256 => Self::hash_blob_impl::<sha2::Sha256>(data),
            Self::Sha512 => Self::hash_blob_impl::<sha2::Sha512>(data),
        }
    }

    /// Hash a blob using a digest.
    fn hash_blob_impl<D>(data: &[u8]) -> String
    where
        D: Digest,
        digest::Output<D>: std::fmt::LowerHex,
    {
        // Compute the hash of the contents.
        let mut digest = D::new();
        digest.update(data);
        format!("{:x}", digest.finalize())
    }

    /// The name of the algorithm.
    pub fn name(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }
}

/// A reference to a blob in some persistence store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobReference {
    algo: ContentHash,
    hash: String,
}

impl BlobReference {
    /// Compute a blob reference for a given blob.
    pub fn for_blob(blob: &Blob, algo: ContentHash) -> Self {
        let hash = algo.hash_blob(blob);

        Self {
            algo,
            hash,
        }
    }

    /// The algorithm of the blob reference.
    pub fn algo(&self) -> ContentHash {
        self.algo
    }

    /// The hash of the blob reference.
    pub fn hash(&self) -> &str {
        &self.hash
    }
}

/// A binary blob.
///
/// Intended to be stored in a content-addressed storage location.
#[derive(Debug, Clone)]
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Create a new blob from bytes.
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
        }
    }
}

impl std::ops::Deref for Blob {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
