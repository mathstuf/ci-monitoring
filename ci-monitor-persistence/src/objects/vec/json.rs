// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any;
use std::collections::BTreeMap;
use std::fmt::Debug;

use chrono::{DateTime, Utc};
use ci_monitor_core::data::{
    BlobReference, ContentHash, Deployment, DeploymentStatus, Environment, EnvironmentState,
    EnvironmentTier, Instance, Job, JobState, PipelineVariable, PipelineVariableType,
    PipelineVariables,
};
use serde::{Deserialize, Serialize};

use super::{VecIndex, VecLookup, VecStoreError};

fn invalid_enum_string<T>(value: &str) -> VecStoreError {
    VecStoreError::InvalidEnumString {
        typename: std::any::type_name::<T>(),
        value: value.into(),
    }
}

fn enum_to_string_opt<T>(lut: &[(T, &'static str)], en: T) -> Option<&'static str>
where
    T: Debug,
    T: PartialEq<T>,
{
    for (e, s) in lut {
        if *e == en {
            return Some(s);
        }
    }

    None
}

fn enum_to_string<T>(lut: &[(T, &'static str)], en: T) -> &'static str
where
    T: Copy + Debug,
    T: PartialEq<T>,
{
    if let Some(s) = enum_to_string_opt(lut, en) {
        s
    } else {
        panic!(
            "unexpected enum value for {}: {:?}",
            any::type_name::<T>(),
            en,
        );
    }
}

fn enum_from_string<T>(lut: &[(T, &'static str)], st: &str) -> Result<T, VecStoreError>
where
    T: Copy,
    T: PartialEq<T>,
{
    for (e, s) in lut {
        if *s == st {
            return Ok(*e);
        }
    }

    Err(invalid_enum_string::<T>(st))
}

pub(super) trait JsonConvert<T>: for<'a> Deserialize<'a> + Serialize {
    fn convert_to_json(o: &T) -> Self;
    fn create_from_json(&self) -> Result<T, VecStoreError>;
}

#[derive(Deserialize, Serialize)]
pub(super) struct DeploymentJson {
    pipeline: usize,
    environment: usize,
    forge_id: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    status: String,

    cim_fetched_at: DateTime<Utc>,
    cim_refreshed_at: DateTime<Utc>,
}

const DEPLOYMENT_STATUS_TABLE: &[(DeploymentStatus, &str)] = &[
    (DeploymentStatus::Created, "created"),
    (DeploymentStatus::Running, "running"),
    (DeploymentStatus::Success, "success"),
    (DeploymentStatus::Failed, "failed"),
    (DeploymentStatus::Canceled, "canceled"),
    (DeploymentStatus::Blocked, "blocked"),
];

impl JsonConvert<Deployment<VecLookup>> for DeploymentJson {
    fn convert_to_json(o: &Deployment<VecLookup>) -> Self {
        Self {
            pipeline: o.pipeline.idx,
            environment: o.environment.idx,
            forge_id: o.forge_id,
            created_at: o.created_at,
            updated_at: o.updated_at,
            finished_at: o.finished_at,
            status: enum_to_string(DEPLOYMENT_STATUS_TABLE, o.status).into(),
            cim_fetched_at: o.cim_fetched_at,
            cim_refreshed_at: o.cim_refreshed_at,
        }
    }

    fn create_from_json(&self) -> Result<Deployment<VecLookup>, VecStoreError> {
        let mut deployment = Deployment::builder()
            .pipeline(VecIndex::new(self.pipeline))
            .environment(VecIndex::new(self.environment))
            .forge_id(self.forge_id)
            .created_at(self.created_at)
            .updated_at(self.updated_at)
            .status(enum_from_string(DEPLOYMENT_STATUS_TABLE, &self.status)?)
            .build()
            .unwrap();
        deployment.finished_at = self.finished_at;
        deployment.cim_fetched_at = self.cim_fetched_at;
        deployment.cim_refreshed_at = self.cim_refreshed_at;

        Ok(deployment)
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct EnvironmentJson {
    name: String,
    external_url: String,
    state: String,
    tier: String,
    forge_id: u64,
    project: usize,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,

    auto_stop_at: Option<DateTime<Utc>>,
    cim_fetched_at: DateTime<Utc>,
    cim_refreshed_at: DateTime<Utc>,
}

const ENVIRONMENT_STATE_TABLE: &[(EnvironmentState, &str)] = &[
    (EnvironmentState::Available, "available"),
    (EnvironmentState::Stopping, "stopping"),
    (EnvironmentState::Stopped, "stopped"),
];

const ENVIRONMENT_TIER_TABLE: &[(EnvironmentTier, &str)] = &[
    (EnvironmentTier::Production, "production"),
    (EnvironmentTier::Staging, "staging"),
    (EnvironmentTier::Testing, "testing"),
    (EnvironmentTier::Development, "development"),
    (EnvironmentTier::Other, "other"),
];

impl JsonConvert<Environment<VecLookup>> for EnvironmentJson {
    fn convert_to_json(o: &Environment<VecLookup>) -> Self {
        Self {
            name: o.name.clone(),
            external_url: o.external_url.clone(),
            state: enum_to_string(ENVIRONMENT_STATE_TABLE, o.state).into(),
            tier: enum_to_string(ENVIRONMENT_TIER_TABLE, o.tier).into(),
            forge_id: o.forge_id,
            project: o.project.idx,
            created_at: o.created_at,
            updated_at: o.updated_at,
            auto_stop_at: o.auto_stop_at,
            cim_fetched_at: o.cim_fetched_at,
            cim_refreshed_at: o.cim_refreshed_at,
        }
    }

    fn create_from_json(&self) -> Result<Environment<VecLookup>, VecStoreError> {
        let mut environment = Environment::builder()
            .name(&self.name)
            .state(enum_from_string(ENVIRONMENT_STATE_TABLE, &self.state)?)
            .tier(enum_from_string(ENVIRONMENT_TIER_TABLE, &self.tier)?)
            .forge_id(self.forge_id)
            .project(VecIndex::new(self.project))
            .created_at(self.created_at)
            .updated_at(self.updated_at)
            .build()
            .unwrap();
        environment.auto_stop_at = self.auto_stop_at;
        environment.cim_fetched_at = self.cim_fetched_at;
        environment.cim_refreshed_at = self.cim_refreshed_at;

        Ok(environment)
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct InstanceJson {
    unique_id: u64,
    forge: String,
    url: String,
}

impl JsonConvert<Instance> for InstanceJson {
    fn convert_to_json(o: &Instance) -> Self {
        Self {
            unique_id: o.unique_id,
            forge: o.forge.clone(),
            url: o.url.clone(),
        }
    }

    fn create_from_json(&self) -> Result<Instance, VecStoreError> {
        Ok(Instance::builder()
            .unique_id(self.unique_id)
            .forge(&self.forge)
            .url(&self.url)
            .build()
            .unwrap())
    }
}

#[derive(Deserialize, Serialize)]
struct PipelineVariableJson {
    value: String,
    type_: String,
    protected: bool,
    environment: Option<String>,
}

const PIPELINE_VARIABLE_TYPE_TABLE: &[(PipelineVariableType, &str)] = &[
    (PipelineVariableType::File, "file"),
    (PipelineVariableType::String, "string"),
];

impl JsonConvert<PipelineVariable> for PipelineVariableJson {
    fn convert_to_json(o: &PipelineVariable) -> Self {
        Self {
            value: o.value.clone(),
            type_: enum_to_string(PIPELINE_VARIABLE_TYPE_TABLE, o.type_).into(),
            protected: o.protected,
            environment: o.environment.clone(),
        }
    }

    fn create_from_json(&self) -> Result<PipelineVariable, VecStoreError> {
        let mut pipeline_variable = PipelineVariable::builder()
            .value(&self.value)
            .type_(enum_from_string(PIPELINE_VARIABLE_TYPE_TABLE, &self.type_)?)
            .build()
            .unwrap();
        pipeline_variable.protected = self.protected;
        pipeline_variable.environment.clone_from(&self.environment);

        Ok(pipeline_variable)
    }
}

#[derive(Deserialize, Serialize)]
struct PipelineVariablesJson {
    variables: BTreeMap<String, PipelineVariableJson>,
}

impl JsonConvert<PipelineVariables> for PipelineVariablesJson {
    fn convert_to_json(o: &PipelineVariables) -> Self {
        Self {
            variables: o
                .variables
                .iter()
                .map(|(k, v)| (k.clone(), PipelineVariableJson::convert_to_json(v)))
                .collect(),
        }
    }

    fn create_from_json(&self) -> Result<PipelineVariables, VecStoreError> {
        self.variables
            .iter()
            .map(|(k, v)| Ok((k.clone(), v.create_from_json()?)))
            .collect()
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct JobJson {
    user: usize,
    name: String,
    stage: String,
    allow_failure: bool,
    tags: Vec<String>,
    variables: PipelineVariablesJson,
    state: String,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    erased_at: Option<DateTime<Utc>>,
    queued_duration: Option<f64>,
    runner: Option<usize>,
    deployment: Option<usize>,
    forge_id: u64,
    archived: bool,
    url: String,
    pipeline: usize,
    coverage: Option<f64>,
    cim_fetched_at: DateTime<Utc>,
    cim_refreshed_at: DateTime<Utc>,
}

const JOB_STATE_TABLE: &[(JobState, &str)] = &[
    (JobState::Created, "created"),
    (JobState::Pending, "pending"),
    (JobState::Running, "running"),
    (JobState::Failed, "failed"),
    (JobState::Success, "success"),
    (JobState::Canceled, "canceled"),
    (JobState::Skipped, "skipped"),
    (JobState::WaitingForResource, "waiting_for_resource"),
    (JobState::Manual, "manual"),
    (JobState::Scheduled, "scheduled"),
];

impl JsonConvert<Job<VecLookup>> for JobJson {
    fn convert_to_json(o: &Job<VecLookup>) -> Self {
        Self {
            name: o.name.clone(),
            stage: o.stage.clone(),
            allow_failure: o.allow_failure,
            user: o.user.idx,
            tags: o.tags.clone(),
            variables: PipelineVariablesJson::convert_to_json(&o.variables),
            state: enum_to_string(JOB_STATE_TABLE, o.state).into(),
            created_at: o.created_at,
            started_at: o.started_at,
            finished_at: o.finished_at,
            erased_at: o.erased_at,
            queued_duration: o.queued_duration,
            runner: o.runner.map(|r| r.idx),
            deployment: o.deployment.map(|d| d.idx),
            forge_id: o.forge_id,
            archived: o.archived,
            url: o.url.clone(),
            pipeline: o.pipeline.idx,
            coverage: o.coverage,
            cim_fetched_at: o.cim_fetched_at,
            cim_refreshed_at: o.cim_refreshed_at,
        }
    }

    fn create_from_json(&self) -> Result<Job<VecLookup>, VecStoreError> {
        let mut job = Job::builder()
            .user(VecIndex::new(self.user))
            .state(enum_from_string(JOB_STATE_TABLE, &self.state)?)
            .created_at(self.created_at)
            .forge_id(self.forge_id)
            .pipeline(VecIndex::new(self.pipeline))
            .build()
            .unwrap();
        job.name.clone_from(&self.name);
        job.stage.clone_from(&self.stage);
        job.allow_failure = self.allow_failure;
        job.tags.clone_from(&self.tags);
        job.variables = self.variables.create_from_json()?;
        job.started_at = self.started_at;
        job.finished_at = self.finished_at;
        job.erased_at = self.erased_at;
        job.queued_duration = self.queued_duration;
        job.runner = self.runner.map(VecIndex::new);
        job.deployment = self.deployment.map(VecIndex::new);
        job.archived = self.archived;
        job.url.clone_from(&self.url);
        job.coverage = self.coverage;
        job.cim_fetched_at = self.cim_fetched_at;
        job.cim_refreshed_at = self.cim_refreshed_at;

        Ok(job)
    }
}

#[derive(Deserialize, Serialize)]
struct BlobReferenceJson {
    algo: String,
    hash: String,
}

const CONTENT_HASH_TABLE: &[(ContentHash, &str)] = &[
    (ContentHash::Sha256, "sha256"),
    (ContentHash::Sha512, "sha512"),
];

impl JsonConvert<BlobReference> for BlobReferenceJson {
    fn convert_to_json(o: &BlobReference) -> Self {
        Self {
            algo: enum_to_string(CONTENT_HASH_TABLE, o.algo()).into(),
            hash: o.hash().into(),
        }
    }

    fn create_from_json(&self) -> Result<BlobReference, VecStoreError> {
        Ok(BlobReference::new(
            enum_from_string(CONTENT_HASH_TABLE, &self.algo)?,
            self.hash.clone(),
        ))
    }
}
