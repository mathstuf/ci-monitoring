// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use perfect_derive::perfect_derive;

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
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[builder(pattern = "owned")]
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
    #[builder(default)]
    pub name: Option<String>,

    // Repository metadata.
    /// The project the pipeline is associated with.
    pub project: <L as Lookup<Project<L>>>::Index,
    /// The commit the pipeline is building.
    #[builder(setter(into))]
    pub sha: String,
    /// The previous commit the pipeline built.
    #[builder(default, setter(into))]
    pub previous_sha: Option<String>,
    /// The refname the pipeline is building.
    #[builder(default, setter(into))]
    pub refname: Option<String>,
    /// The stable refname for the pipeline.
    #[builder(default, setter(into))]
    pub stable_refname: Option<String>,

    // Execution metadata.
    /// The reason the pipeline was created.
    pub source: PipelineSource,
    /// The schedule which triggered the pipeline.
    #[builder(default)]
    pub schedule: Option<<L as Lookup<PipelineSchedule<L>>>::Index>,
    /// The parent pipeline.
    #[builder(default)]
    pub parent_pipeline: Option<<L as Lookup<Pipeline<L>>>::Index>,
    /// The merge request associated with a pipeline.
    #[builder(default)]
    pub merge_request: Option<<L as Lookup<MergeRequest<L>>>::Index>,
    /// Variables for the pipeline.
    #[builder(default)]
    pub variables: PipelineVariables,
    /// The user that created the pipeline.
    #[builder(default)]
    pub user: Option<<L as Lookup<User<L>>>::Index>,

    // Pipeline results.
    /// The status of the pipeline.
    pub status: PipelineStatus,
    /// The code coverage reported by the pipeline.
    #[builder(default)]
    pub coverage: Option<f64>,

    // Forge metadata.
    /// The ID of the pipeline.
    pub forge_id: u64,
    /// The URL of the pipeline webpage.
    #[builder(setter(into))]
    pub url: String,
    /// Whether the pipeline is archived or not.
    ///
    /// Archived pipelines are essentially read-only.
    #[builder(default)]
    pub archived: bool,
    /// When the pipeline was created.
    pub created_at: DateTime<Utc>,
    /// When the pipeline was last updated.
    pub updated_at: DateTime<Utc>,
    /// When the pipeline started.
    #[builder(default)]
    pub started_at: Option<DateTime<Utc>>,
    /// When the pipeline completed.
    #[builder(default)]
    pub finished_at: Option<DateTime<Utc>>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl<L> Pipeline<L>
where
    L: Lookup<Instance>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<Pipeline<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    /// Create a builder for the structure.
    pub fn builder() -> PipelineBuilder<L> {
        PipelineBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::data::{
        Instance, Pipeline, PipelineBuilderError, PipelineSource, PipelineStatus, Project,
    };
    use crate::Lookup;

    use crate::test::TestLookup;

    fn project(lookup: &mut TestLookup) -> Project<TestLookup> {
        let instance = Instance::builder()
            .unique_id(0)
            .forge("forge")
            .url("url")
            .build()
            .unwrap();
        let idx = lookup.store(instance);

        Project::builder()
            .forge_id(0)
            .instance(idx)
            .build()
            .unwrap()
    }

    #[test]
    fn project_is_required() {
        let err = Pipeline::<TestLookup>::builder()
            .sha("0000000000000000000000000000000000000000")
            .source(PipelineSource::Schedule)
            .status(PipelineStatus::Created)
            .forge_id(0)
            .url("url")
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineBuilderError, "project");
    }

    #[test]
    fn sha_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Pipeline::<TestLookup>::builder()
            .project(proj_idx)
            .source(PipelineSource::Schedule)
            .status(PipelineStatus::Created)
            .forge_id(0)
            .url("url")
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineBuilderError, "sha");
    }

    #[test]
    fn source_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Pipeline::<TestLookup>::builder()
            .project(proj_idx)
            .sha("0000000000000000000000000000000000000000")
            .status(PipelineStatus::Created)
            .forge_id(0)
            .url("url")
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineBuilderError, "source");
    }

    #[test]
    fn status_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Pipeline::<TestLookup>::builder()
            .project(proj_idx)
            .sha("0000000000000000000000000000000000000000")
            .source(PipelineSource::Schedule)
            .forge_id(0)
            .url("url")
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineBuilderError, "status");
    }

    #[test]
    fn forge_id_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Pipeline::<TestLookup>::builder()
            .project(proj_idx)
            .sha("0000000000000000000000000000000000000000")
            .source(PipelineSource::Schedule)
            .status(PipelineStatus::Created)
            .url("url")
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineBuilderError, "forge_id");
    }

    #[test]
    fn url_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Pipeline::<TestLookup>::builder()
            .project(proj_idx)
            .sha("0000000000000000000000000000000000000000")
            .source(PipelineSource::Schedule)
            .status(PipelineStatus::Created)
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineBuilderError, "url");
    }

    #[test]
    fn created_at_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Pipeline::<TestLookup>::builder()
            .project(proj_idx)
            .sha("0000000000000000000000000000000000000000")
            .source(PipelineSource::Schedule)
            .status(PipelineStatus::Created)
            .forge_id(0)
            .url("url")
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineBuilderError, "created_at");
    }

    #[test]
    fn updated_at_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Pipeline::<TestLookup>::builder()
            .project(proj_idx)
            .sha("0000000000000000000000000000000000000000")
            .source(PipelineSource::Schedule)
            .status(PipelineStatus::Created)
            .forge_id(0)
            .url("url")
            .created_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineBuilderError, "updated_at");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        Pipeline::<TestLookup>::builder()
            .project(proj_idx)
            .sha("0000000000000000000000000000000000000000")
            .source(PipelineSource::Schedule)
            .status(PipelineStatus::Created)
            .forge_id(0)
            .url("url")
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap();
    }
}
