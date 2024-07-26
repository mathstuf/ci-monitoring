// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ci_monitor_core::data::{
    Deployment, Environment, Instance, Job, JobArtifact, MergeRequest, Pipeline, PipelineSchedule,
    Project, Runner, RunnerHost, User,
};

use super::json::{self, JsonConvert};
use super::{VecIndex, VecLookup, VecStoreError};

trait Typename {
    fn typename() -> &'static str;
}

macro_rules! impl_typename {
    ($t:ty, $name:expr) => {
        impl Typename for $t {
            fn typename() -> &'static str {
                $name
            }
        }
    };
}

impl_typename!(Deployment<VecLookup>, "deployment");
impl_typename!(Environment<VecLookup>, "environment");
impl_typename!(Instance, "instance");
impl_typename!(Job<VecLookup>, "job");
impl_typename!(JobArtifact<VecLookup>, "job artifact");
impl_typename!(MergeRequest<VecLookup>, "merge request");
impl_typename!(Pipeline<VecLookup>, "pipeline");
impl_typename!(PipelineSchedule<VecLookup>, "pipeline schedule");
impl_typename!(Project<VecLookup>, "project");
impl_typename!(Runner<VecLookup>, "runner");
impl_typename!(RunnerHost, "runner host");
impl_typename!(User<VecLookup>, "user");

pub(super) trait JsonStorable: Sized {
    type Json: JsonConvert<Self>;

    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        let json = Self::Json::convert_to_json(self);
        serde_json::to_value(json)
    }

    fn from_json(json: serde_json::Value) -> Result<Self, VecStoreError> {
        let value: Self::Json = serde_json::from_value(json)?;
        value.create_from_json()
    }

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        let _ = self_index;
        let _ = storage;
        Ok(())
    }
}

#[allow(clippy::ptr_arg)] // Ensure we're dealing with the entire set of entities.
fn validate_index<T, F>(
    from_index: &VecIndex<F>,
    storage: &Vec<T>,
    index: &VecIndex<T>,
) -> Result<(), VecStoreError>
where
    T: Typename,
    F: Typename,
{
    if storage.len() < index.idx {
        return Err(VecStoreError::MissingIndex {
            missing_type: T::typename(),
            missing_index: index.idx,
            from_type: F::typename(),
            from_index: from_index.idx,
        });
    }

    Ok(())
}

impl JsonStorable for Deployment<VecLookup> {
    type Json = json::DeploymentJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.pipelines, &self.pipeline)?;
        validate_index(&self_index, &storage.environments, &self.environment)?;

        Ok(())
    }
}

impl JsonStorable for Environment<VecLookup> {
    type Json = json::EnvironmentJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.projects, &self.project)?;

        Ok(())
    }
}

impl JsonStorable for Instance {
    type Json = json::InstanceJson;
}

impl JsonStorable for Job<VecLookup> {
    type Json = json::JobJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.pipelines, &self.pipeline)?;
        validate_index(&self_index, &storage.users, &self.user)?;
        if let Some(runner) = self.runner.as_ref() {
            validate_index(&self_index, &storage.runners, runner)?;
        }
        if let Some(deployment) = self.deployment.as_ref() {
            validate_index(&self_index, &storage.deployments, deployment)?;
        }

        Ok(())
    }
}

impl JsonStorable for JobArtifact<VecLookup> {
    type Json = json::JobArtifactJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.jobs, &self.job)?;

        Ok(())
    }
}

impl JsonStorable for MergeRequest<VecLookup> {
    type Json = json::MergeRequestJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.projects, &self.source_project)?;
        validate_index(&self_index, &storage.projects, &self.target_project)?;
        validate_index(&self_index, &storage.users, &self.author)?;

        Ok(())
    }
}

impl JsonStorable for Pipeline<VecLookup> {
    type Json = json::PipelineJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.projects, &self.project)?;
        if let Some(schedule) = self.schedule.as_ref() {
            validate_index(&self_index, &storage.pipeline_schedules, schedule)?;
        }
        if let Some(parent_pipeline) = self.parent_pipeline.as_ref() {
            validate_index(&self_index, &storage.pipelines, parent_pipeline)?;
        }
        if let Some(merge_request) = self.merge_request.as_ref() {
            validate_index(&self_index, &storage.merge_requests, merge_request)?;
        }
        if let Some(user) = self.user.as_ref() {
            validate_index(&self_index, &storage.users, user)?;
        }

        Ok(())
    }
}

impl JsonStorable for PipelineSchedule<VecLookup> {
    type Json = json::PipelineScheduleJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.projects, &self.project)?;
        validate_index(&self_index, &storage.users, &self.owner)?;

        Ok(())
    }
}

impl JsonStorable for Project<VecLookup> {
    type Json = json::ProjectJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.instances, &self.instance)?;

        Ok(())
    }
}

impl JsonStorable for Runner<VecLookup> {
    type Json = json::RunnerJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.instances, &self.instance)?;
        if let Some(runner_host) = self.runner_host.as_ref() {
            validate_index(&self_index, &storage.runner_hosts, runner_host)?;
        }
        for project in &self.projects {
            validate_index(&self_index, &storage.projects, project)?;
        }

        Ok(())
    }
}

impl JsonStorable for RunnerHost {
    type Json = json::RunnerHostJson;
}

impl JsonStorable for User<VecLookup> {
    type Json = json::UserJson;

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        validate_index(&self_index, &storage.instances, &self.instance)?;

        Ok(())
    }
}
