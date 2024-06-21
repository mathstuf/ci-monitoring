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
mod merge_request;
mod pipeline;
mod pipeline_schedule;
mod pipeline_variables;
mod project;
mod runner_host;
mod user;

pub use blob::Blob;
pub use blob::BlobReference;
pub use blob::ContentHash;

pub use instance::Instance;

pub use job_artifact::ArtifactExpiration;
pub use job_artifact::ArtifactKind;
pub use job_artifact::ArtifactState;
pub use job_artifact::JobArtifact;

pub use merge_request::MergeRequest;
pub use merge_request::MergeRequestStatus;

pub use pipeline::Pipeline;
pub use pipeline::PipelineSource;
pub use pipeline::PipelineStatus;

pub use pipeline_schedule::PipelineSchedule;

pub use pipeline_variables::PipelineVariable;
pub use pipeline_variables::PipelineVariableType;
pub use pipeline_variables::PipelineVariables;

pub use project::Project;

pub use runner_host::RunnerHost;

pub use user::User;
