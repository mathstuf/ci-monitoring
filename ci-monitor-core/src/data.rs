// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Data structures.
//!
//! With some convenience methods for managing them.

mod blob;
mod instance;
mod job_artifact;
mod project;

pub use blob::Blob;
pub use blob::BlobReference;
pub use blob::ContentHash;

pub use instance::Instance;

pub use job_artifact::ArtifactExpiration;
pub use job_artifact::ArtifactKind;
pub use job_artifact::ArtifactState;
pub use job_artifact::JobArtifact;

pub use project::Project;
