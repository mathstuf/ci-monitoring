// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Data structures.
//!
//! With some convenience methods for managing them.

mod blob;
mod deployment;
mod environment;
mod instance;
mod job;
mod job_artifact;
mod merge_request;
mod pipeline;
mod pipeline_schedule;
mod pipeline_variables;
mod project;
mod runner;
mod runner_host;
mod user;

pub use blob::Blob;
pub use blob::BlobReference;
pub use blob::ContentHash;

pub use deployment::Deployment;
pub use deployment::DeploymentStatus;

pub use environment::Environment;
pub use environment::EnvironmentState;
pub use environment::EnvironmentTier;

pub use instance::Instance;
pub use instance::InstanceBuilder;
pub use instance::InstanceBuilderError;

pub use job::Job;
pub use job::JobState;

pub use job_artifact::ArtifactExpiration;
pub use job_artifact::ArtifactKind;
pub use job_artifact::ArtifactState;
pub use job_artifact::JobArtifact;

pub use merge_request::MergeRequest;
pub use merge_request::MergeRequestBuilder;
pub use merge_request::MergeRequestBuilderError;
pub use merge_request::MergeRequestStatus;

pub use pipeline::Pipeline;
pub use pipeline::PipelineBuilder;
pub use pipeline::PipelineBuilderError;
pub use pipeline::PipelineSource;
pub use pipeline::PipelineStatus;

pub use pipeline_schedule::PipelineSchedule;
pub use pipeline_schedule::PipelineScheduleBuilder;
pub use pipeline_schedule::PipelineScheduleBuilderError;

pub use pipeline_variables::PipelineVariable;
pub use pipeline_variables::PipelineVariableType;
pub use pipeline_variables::PipelineVariables;

pub use project::Project;
pub use project::ProjectBuilder;
pub use project::ProjectBuilderError;

pub use runner::Runner;
pub use runner::RunnerBuilder;
pub use runner::RunnerBuilderError;
pub use runner::RunnerProtectionLevel;
pub use runner::RunnerType;

pub use runner_host::RunnerHost;

pub use user::User;
pub use user::UserBuilder;
pub use user::UserBuilderError;
