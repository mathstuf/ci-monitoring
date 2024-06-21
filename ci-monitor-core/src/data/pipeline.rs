// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};

use crate::data::{Instance, MergeRequest, PipelineSchedule, PipelineVariables, Project, User};
use crate::Lookup;

/// The source of a pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PipelineSource {
    /// Created via the API.
    Api,
    /// Created via a chatbot.
    Chat,
    /// Created via an external event.
    External,
    /// Created via an external pull request event.
    ExternalPullRequestEvent,
    /// Created due to a merge request event.
    MergeRequestEvent,
    /// Created to perform a DAST scan.
    OnDemandDastScan,
    /// Created to perform a DAST validation.
    OnDemandDastValidation,
    /// Created as a child of another pipeline.
    ParentPipeline,
    /// Created through the action of another pipeline.
    Pipeline,
    /// Created due to a push to a ref.
    Push,
    /// Created due to a schedule.
    Schedule,
    /// Created for a security orchestration.
    SecurityOrchestrationPolicy,
    /// Created via a trigger token.
    Trigger,
    /// Created via the web interface.
    Web,
    /// Created via the web IDE.
    WebIde,
}

/// The overall status of a pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PipelineStatus {
    /// The pipeline has been created.
    Created,
    /// The pipeline is waiting for a resource to be available.
    WaitingForResource,
    /// The jobs in the pipeline are being constructed.
    Preparing,
    /// The pipeline is waiting for jobs to be executed.
    Pending,
    /// The pipeline is running.
    Running,
    /// The pipeline has completed successfully.
    Success,
    /// The pipeline has failed.
    Failed,
    /// The pipeline has been canceled.
    Canceled,
    /// The pipeline has been skipped.
    Skipped,
    /// The pipeline is waiting for manual interaction.
    Manual,
    /// The pipeline is scheduled.
    Scheduled,
    /// The pipeline has completed.
    Completed,
    /// The pipeline has completed without success or failure.
    Neutral,
    /// The pipeline is stale.
    Stale,
    /// The pipeline failed to start.
    StartupFailure,
    /// The pipeline has timed out.
    TimedOut,
}

/// A pipeline which performs CI tasks for a project.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Pipeline<L>
where
    L: Lookup<Instance>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<Pipeline<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    // Pipeline metadata.
    /// The name of the pipeline.
    pub name: Option<String>,

    // Repository metadata.
    /// The project the pipeline is associated with.
    pub project: <L as Lookup<Project<L>>>::Index,
    /// The commit the pipeline is building.
    pub sha: String,
    /// The previous commit the pipeline built.
    pub previous_sha: Option<String>,
    /// The refname the pipeline is building.
    pub refname: Option<String>,
    /// The stable refname for the pipeline.
    pub stable_refname: Option<String>,

    // Execution metadata.
    /// The reason the pipeline was created.
    pub source: PipelineSource,
    /// The schedule which triggered the pipeline.
    pub schedule: Option<<L as Lookup<PipelineSchedule<L>>>::Index>,
    /// The parent pipeline.
    pub parent_pipeline: Option<<L as Lookup<Pipeline<L>>>::Index>,
    /// The merge request associated with a pipeline.
    pub merge_request: Option<<L as Lookup<MergeRequest<L>>>::Index>,
    /// Variables for the pipeline.
    pub variables: PipelineVariables,
    /// The user that created the pipeline.
    pub user: <L as Lookup<User<L>>>::Index,

    // Pipeline results.
    /// The status of the pipeline.
    pub status: PipelineStatus,
    /// The code coverage reported by the pipeline.
    pub coverage: Option<f64>,

    // Forge metadata.
    /// The ID of the pipeline.
    pub forge_id: u64,
    /// The URL of the pipeline webpage.
    pub url: String,
    /// Whether the pipeline is archived or not.
    ///
    /// Archived pipelines are essentially read-only.
    pub archived: bool,
    /// When the pipeline was created.
    pub created_at: DateTime<Utc>,
    /// When the pipeline was last updated.
    pub updated_at: DateTime<Utc>,
    /// When the pipeline started.
    pub started_at: Option<DateTime<Utc>>,
    /// When the pipeline completed.
    pub finished_at: Option<DateTime<Utc>>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
