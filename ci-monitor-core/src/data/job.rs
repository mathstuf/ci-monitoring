// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use perfect_derive::perfect_derive;

use crate::data::{
    Deployment, Environment, Instance, MergeRequest, Pipeline, PipelineSchedule, PipelineVariables,
    Project, Runner, RunnerHost, User,
};
use crate::Lookup;

/// The state of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum JobState {
    /// The job was created.
    Created,
    /// The job is waiting for a runner.
    Pending,
    /// The job is running.
    Running,
    /// The job failed.
    Failed,
    /// The job completed successfully.
    Success,
    /// The job was canceled.
    Canceled,
    /// The job was skipped.
    Skipped,
    /// The job is waiting for a resource.
    WaitingForResource,
    /// The job is waiting for manual interaction.
    Manual,
}

/// A job within a pipeline.
#[perfect_derive(Debug, Clone)]
#[non_exhaustive]
pub struct Job<L>
where
    L: Lookup<Deployment<L>>,
    L: Lookup<Environment<L>>,
    L: Lookup<Instance>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<Pipeline<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Project<L>>,
    L: Lookup<Runner<L>>,
    L: Lookup<RunnerHost>,
    L: Lookup<User<L>>,
{
    // Metadata.
    /// The name of the job.
    pub name: String,
    /// The stage of the job within its pipeline.
    pub stage: String,
    /// Whether the job is allowed to fail or not.
    pub allow_failure: bool,
    /// The user that created the job.
    pub user: <L as Lookup<User<L>>>::Index,
    /// The tags of the job.
    pub tags: Vec<String>,
    /// Variables for the job.
    pub variables: PipelineVariables,

    // Runtime metadata.
    /// The state of the job.
    pub state: JobState,
    /// When the job was created.
    pub created_at: DateTime<Utc>,
    /// When the job was started.
    pub started_at: Option<DateTime<Utc>>,
    /// When the job finished.
    pub finished_at: Option<DateTime<Utc>>,
    /// When the job was erased.
    pub erased_at: Option<DateTime<Utc>>,
    /// How long the job was queued.
    pub queued_duration: Option<f64>,
    /// The runner for the job.
    pub runner: Option<<L as Lookup<Runner<L>>>::Index>,
    /// The deployment the job publishes to.
    pub deployment: Option<<L as Lookup<Deployment<L>>>::Index>,

    // Forge metadata.
    /// The ID of the job.
    pub forge_id: u64,
    /// Whether the job is archived or not.
    pub archived: bool,
    /// The URL of the job.
    pub url: String,
    /// The pipeline the job belongs to.
    pub pipeline: <L as Lookup<Pipeline<L>>>::Index,

    // Job outputs.
    /// The coverage reported by the job.
    pub coverage: Option<f64>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
