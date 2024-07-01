// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
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
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[builder(pattern = "owned")]
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
    #[builder(default, setter(into))]
    pub name: String,
    /// The stage of the job within its pipeline.
    #[builder(default, setter(into))]
    pub stage: String,
    /// Whether the job is allowed to fail or not.
    #[builder(default)]
    pub allow_failure: bool,
    /// The user that created the job.
    pub user: <L as Lookup<User<L>>>::Index,
    /// The tags of the job.
    #[builder(default)]
    pub tags: Vec<String>,
    /// Variables for the job.
    #[builder(default)]
    pub variables: PipelineVariables,

    // Runtime metadata.
    /// The state of the job.
    pub state: JobState,
    /// When the job was created.
    pub created_at: DateTime<Utc>,
    /// When the job was started.
    #[builder(default)]
    pub started_at: Option<DateTime<Utc>>,
    /// When the job finished.
    #[builder(default)]
    pub finished_at: Option<DateTime<Utc>>,
    /// When the job was erased.
    #[builder(default)]
    pub erased_at: Option<DateTime<Utc>>,
    /// How long the job was queued.
    #[builder(default)]
    pub queued_duration: Option<f64>,
    /// The runner for the job.
    #[builder(default)]
    pub runner: Option<<L as Lookup<Runner<L>>>::Index>,
    /// The deployment the job publishes to.
    #[builder(default)]
    pub deployment: Option<<L as Lookup<Deployment<L>>>::Index>,

    // Forge metadata.
    /// The ID of the job.
    pub forge_id: u64,
    /// Whether the job is archived or not.
    #[builder(default)]
    pub archived: bool,
    /// The URL of the job.
    #[builder(default, setter(into))]
    pub url: String,
    /// The pipeline the job belongs to.
    pub pipeline: <L as Lookup<Pipeline<L>>>::Index,

    // Job outputs.
    /// The coverage reported by the job.
    #[builder(default)]
    pub coverage: Option<f64>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl<L> Job<L>
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
    /// Create a builder for the structure.
    pub fn builder() -> JobBuilder<L> {
        JobBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::data::{
        Instance, Job, JobBuilderError, JobState, Pipeline, PipelineSource, PipelineStatus,
        Project, User,
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

    fn user(instance: <TestLookup as Lookup<Instance>>::Index) -> User<TestLookup> {
        User::builder()
            .forge_id(0)
            .instance(instance)
            .build()
            .unwrap()
    }

    fn pipeline(
        project: <TestLookup as Lookup<Project<TestLookup>>>::Index,
        user: <TestLookup as Lookup<User<TestLookup>>>::Index,
    ) -> Pipeline<TestLookup> {
        Pipeline::builder()
            .project(project)
            .sha("0000000000000000000000000000000000000000")
            .source(PipelineSource::Schedule)
            .user(user)
            .status(PipelineStatus::Created)
            .forge_id(0)
            .url("url")
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap()
    }

    #[test]
    fn user_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx.clone());
        let pipeline_idx = lookup.store(pipeline);

        let err = Job::<TestLookup>::builder()
            .state(JobState::Created)
            .created_at(Utc::now())
            .forge_id(0)
            .pipeline(pipeline_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobBuilderError, "user");
    }

    #[test]
    fn state_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx.clone());
        let pipeline_idx = lookup.store(pipeline);

        let err = Job::<TestLookup>::builder()
            .user(user_idx)
            .created_at(Utc::now())
            .forge_id(0)
            .pipeline(pipeline_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobBuilderError, "state");
    }

    #[test]
    fn created_at_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx.clone());
        let pipeline_idx = lookup.store(pipeline);

        let err = Job::<TestLookup>::builder()
            .user(user_idx)
            .state(JobState::Created)
            .forge_id(0)
            .pipeline(pipeline_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobBuilderError, "created_at");
    }

    #[test]
    fn forge_id_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx.clone());
        let pipeline_idx = lookup.store(pipeline);

        let err = Job::<TestLookup>::builder()
            .user(user_idx)
            .state(JobState::Created)
            .created_at(Utc::now())
            .pipeline(pipeline_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobBuilderError, "forge_id");
    }

    #[test]
    fn pipeline_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);

        let err = Job::<TestLookup>::builder()
            .user(user_idx)
            .state(JobState::Created)
            .created_at(Utc::now())
            .forge_id(0)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, JobBuilderError, "pipeline");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx.clone());
        let pipeline_idx = lookup.store(pipeline);

        Job::<TestLookup>::builder()
            .user(user_idx)
            .state(JobState::Created)
            .created_at(Utc::now())
            .forge_id(0)
            .pipeline(pipeline_idx)
            .build()
            .unwrap();
    }
}
