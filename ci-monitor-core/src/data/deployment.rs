// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use perfect_derive::perfect_derive;

use crate::data::{Environment, Instance, MergeRequest, Pipeline, PipelineSchedule, Project, User};
use crate::Lookup;

/// The status of a deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeploymentStatus {
    /// The deployment has been created.
    Created,
    /// The deployment is running.
    Running,
    /// The deployment completed successfully.
    Success,
    /// The deployment completed with failure.
    Failed,
    /// The deployment was canceled.
    Canceled,
    /// The deployment is blocked.
    Blocked,
}

/// A deployment into an environment.
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[builder(pattern = "owned")]
#[non_exhaustive]
pub struct Deployment<L>
where
    L: Lookup<Environment<L>>,
    L: Lookup<Instance>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<Pipeline<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    // Project metadata.
    /// The pipeline which created the deployment.
    pub pipeline: <L as Lookup<Pipeline<L>>>::Index,
    /// The environment which was deployed into.
    pub environment: <L as Lookup<Environment<L>>>::Index,

    // Forge metadata.
    /// The ID of the deployment.
    pub forge_id: u64,
    /// When the deployment was created.
    pub created_at: DateTime<Utc>,
    /// When the deployment was updated.
    pub updated_at: DateTime<Utc>,
    /// When the deployment completed.
    #[builder(default)]
    pub finished_at: Option<DateTime<Utc>>,
    /// The status of the deployment.
    pub status: DeploymentStatus,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl<L> Deployment<L>
where
    L: Lookup<Environment<L>>,
    L: Lookup<Instance>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<Pipeline<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    /// Create a builder for the structure.
    pub fn builder() -> DeploymentBuilder<L> {
        DeploymentBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::data::{
        Deployment, DeploymentBuilderError, DeploymentStatus, Environment, EnvironmentState,
        EnvironmentTier, Instance, Pipeline, PipelineSource, PipelineStatus, Project, User,
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

    fn environment(
        project: <TestLookup as Lookup<Project<TestLookup>>>::Index,
    ) -> Environment<TestLookup> {
        Environment::builder()
            .name("name")
            .state(EnvironmentState::Available)
            .tier(EnvironmentTier::Testing)
            .forge_id(0)
            .project(project)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap()
    }

    #[test]
    fn pipeline_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);
        let environment = environment(proj_idx);
        let environment_idx = lookup.store(environment);

        let err = Deployment::<TestLookup>::builder()
            .environment(environment_idx)
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .status(DeploymentStatus::Created)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, DeploymentBuilderError, "pipeline");
    }

    #[test]
    fn environment_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx);
        let pipeline_idx = lookup.store(pipeline);

        let err = Deployment::<TestLookup>::builder()
            .pipeline(pipeline_idx)
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .status(DeploymentStatus::Created)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, DeploymentBuilderError, "environment");
    }

    #[test]
    fn forge_id_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx);
        let environment = environment(proj_idx);
        let pipeline_idx = lookup.store(pipeline);
        let environment_idx = lookup.store(environment);

        let err = Deployment::<TestLookup>::builder()
            .pipeline(pipeline_idx)
            .environment(environment_idx)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .status(DeploymentStatus::Created)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, DeploymentBuilderError, "forge_id");
    }

    #[test]
    fn created_at_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx);
        let environment = environment(proj_idx);
        let pipeline_idx = lookup.store(pipeline);
        let environment_idx = lookup.store(environment);

        let err = Deployment::<TestLookup>::builder()
            .pipeline(pipeline_idx)
            .environment(environment_idx)
            .forge_id(0)
            .updated_at(Utc::now())
            .status(DeploymentStatus::Created)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, DeploymentBuilderError, "created_at");
    }

    #[test]
    fn updated_at_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx);
        let environment = environment(proj_idx);
        let pipeline_idx = lookup.store(pipeline);
        let environment_idx = lookup.store(environment);

        let err = Deployment::<TestLookup>::builder()
            .pipeline(pipeline_idx)
            .environment(environment_idx)
            .forge_id(0)
            .created_at(Utc::now())
            .status(DeploymentStatus::Created)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, DeploymentBuilderError, "updated_at");
    }

    #[test]
    fn status_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx);
        let environment = environment(proj_idx);
        let pipeline_idx = lookup.store(pipeline);
        let environment_idx = lookup.store(environment);

        let err = Deployment::<TestLookup>::builder()
            .pipeline(pipeline_idx)
            .environment(environment_idx)
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, DeploymentBuilderError, "status");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);
        let proj_idx = lookup.store(proj);
        let pipeline = pipeline(proj_idx.clone(), user_idx);
        let environment = environment(proj_idx);
        let pipeline_idx = lookup.store(pipeline);
        let environment_idx = lookup.store(environment);

        Deployment::<TestLookup>::builder()
            .pipeline(pipeline_idx)
            .environment(environment_idx)
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .status(DeploymentStatus::Created)
            .build()
            .unwrap();
    }
}
