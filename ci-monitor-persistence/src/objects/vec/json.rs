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
    ArtifactExpiration, ArtifactKind, ArtifactState, BlobReference, ContentHash, Deployment,
    DeploymentStatus, Environment, EnvironmentState, EnvironmentTier, Instance, Job, JobArtifact,
    JobState, MergeRequest, MergeRequestStatus, Pipeline, PipelineSchedule, PipelineSource,
    PipelineStatus, PipelineVariable, PipelineVariableType, PipelineVariables, Project,
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

#[derive(Deserialize, Serialize)]
pub(super) struct JobArtifactJson {
    state: String,
    kind: String,
    expire_at: String,
    name: String,
    blob: Option<BlobReferenceJson>,
    size: u64,
    unique_id: u64,
    job: usize,
}

const ARTIFACT_EXPIRATION_TABLE: &[(ArtifactExpiration, &str)] = &[
    (ArtifactExpiration::Unknown, "unknown"),
    (ArtifactExpiration::Never, "never"),
];

fn artifact_expiration_to_string(ae: ArtifactExpiration) -> String {
    if let ArtifactExpiration::At(dt) = ae {
        let mut s = Vec::new();
        {
            let mut ser = serde_json::Serializer::new(&mut s);
            dt.serialize(&mut ser).unwrap();
        }
        String::from_utf8_lossy(&s).into_owned()
    } else {
        enum_to_string(ARTIFACT_EXPIRATION_TABLE, ae).into()
    }
}

fn artifact_expiration_from_string(s: &str) -> Result<ArtifactExpiration, VecStoreError> {
    if let Ok(ae) = enum_from_string(ARTIFACT_EXPIRATION_TABLE, s) {
        Ok(ae)
    } else {
        let mut des = serde_json::Deserializer::from_str(s);
        let dt = DateTime::<Utc>::deserialize(&mut des)?;
        Ok(ArtifactExpiration::At(dt))
    }
}

const ARTIFACT_STATE_TABLE: &[(ArtifactState, &str)] = &[
    (ArtifactState::Unknown, "unknown"),
    (ArtifactState::Pending, "pending"),
    (ArtifactState::Expired, "expired"),
    (ArtifactState::Present, "present"),
    (ArtifactState::Stored, "stored"),
];

impl JsonConvert<JobArtifact<VecLookup>> for JobArtifactJson {
    fn convert_to_json(o: &JobArtifact<VecLookup>) -> Self {
        Self {
            state: enum_to_string(ARTIFACT_STATE_TABLE, o.state).into(),
            kind: o.kind.as_str().into(),
            expire_at: artifact_expiration_to_string(o.expire_at),
            name: o.name.clone(),
            blob: o.blob.as_ref().map(BlobReferenceJson::convert_to_json),
            size: o.size,
            unique_id: o.unique_id,
            job: o.job.idx,
        }
    }

    fn create_from_json(&self) -> Result<JobArtifact<VecLookup>, VecStoreError> {
        let mut job_artifact = JobArtifact::builder()
            .kind(
                ArtifactKind::parse(&self.kind)
                    .ok_or_else(|| invalid_enum_string::<ArtifactKind>(&self.kind))?,
            )
            .name(&self.name)
            .size(self.size)
            .unique_id(self.unique_id)
            .job(VecIndex::new(self.job))
            .build()
            .unwrap();
        job_artifact.state = enum_from_string(ARTIFACT_STATE_TABLE, &self.state)?;
        job_artifact.expire_at = artifact_expiration_from_string(&self.expire_at)?;
        job_artifact.blob = self
            .blob
            .as_ref()
            .map(BlobReferenceJson::create_from_json)
            .transpose()?;

        Ok(job_artifact)
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct MergeRequestJson {
    id: u64,
    source_project: usize,
    source_branch: String,
    sha: String,
    target_project: usize,
    target_branch: String,
    forge_id: u64,
    title: String,
    description: String,
    state: String,
    author: usize,
    url: String,
    cim_fetched_at: DateTime<Utc>,
    cim_refreshed_at: DateTime<Utc>,
}

const MERGE_REQUEST_STATUS_TABLE: &[(MergeRequestStatus, &str)] = &[
    (MergeRequestStatus::Open, "open"),
    (MergeRequestStatus::Closed, "closed"),
    (MergeRequestStatus::Merged, "merged"),
];

impl JsonConvert<MergeRequest<VecLookup>> for MergeRequestJson {
    fn convert_to_json(o: &MergeRequest<VecLookup>) -> Self {
        Self {
            id: o.id,
            source_project: o.source_project.idx,
            source_branch: o.source_branch.clone(),
            sha: o.sha.clone(),
            target_project: o.target_project.idx,
            target_branch: o.target_branch.clone(),
            forge_id: o.forge_id,
            title: o.title.clone(),
            description: o.description.clone(),
            state: enum_to_string(MERGE_REQUEST_STATUS_TABLE, o.state).into(),
            author: o.author.idx,
            url: o.url.clone(),
            cim_fetched_at: o.cim_fetched_at,
            cim_refreshed_at: o.cim_refreshed_at,
        }
    }

    fn create_from_json(&self) -> Result<MergeRequest<VecLookup>, VecStoreError> {
        let mut merge_request = MergeRequest::builder()
            .id(self.id)
            .source_project(VecIndex::new(self.source_project))
            .target_project(VecIndex::new(self.target_project))
            .forge_id(self.forge_id)
            .state(enum_from_string(MERGE_REQUEST_STATUS_TABLE, &self.state)?)
            .author(VecIndex::new(self.author))
            .url(&self.url)
            .build()
            .unwrap();
        merge_request.source_branch.clone_from(&self.source_branch);
        merge_request.sha.clone_from(&self.sha);
        merge_request.target_branch.clone_from(&self.target_branch);
        merge_request.title.clone_from(&self.title);
        merge_request.description.clone_from(&self.description);
        merge_request.cim_fetched_at = self.cim_fetched_at;
        merge_request.cim_refreshed_at = self.cim_refreshed_at;

        Ok(merge_request)
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct PipelineJson {
    name: Option<String>,
    project: usize,
    sha: String,
    previous_sha: Option<String>,
    refname: Option<String>,
    stable_refname: Option<String>,
    source: String,
    schedule: Option<usize>,
    parent_pipeline: Option<usize>,
    merge_request: Option<usize>,
    variables: PipelineVariablesJson,
    user: Option<usize>,
    status: String,
    coverage: Option<f64>,
    forge_id: u64,
    url: String,
    archived: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    cim_fetched_at: DateTime<Utc>,
    cim_refreshed_at: DateTime<Utc>,
}

const PIPELINE_SOURCE_TABLE: &[(PipelineSource, &str)] = &[
    (PipelineSource::Api, "api"),
    (PipelineSource::Chat, "chat"),
    (PipelineSource::External, "external"),
    (
        PipelineSource::ExternalPullRequestEvent,
        "external_pull_request_event",
    ),
    (PipelineSource::MergeRequestEvent, "merge_request_event"),
    (PipelineSource::OnDemandDastScan, "on_demand_dast_scan"),
    (
        PipelineSource::OnDemandDastValidation,
        "on_demand_dast_validation",
    ),
    (PipelineSource::ParentPipeline, "parent_pipeline"),
    (PipelineSource::Pipeline, "pipeline"),
    (PipelineSource::Push, "push"),
    (PipelineSource::Schedule, "schedule"),
    (
        PipelineSource::SecurityOrchestrationPolicy,
        "security_orchestration_policy",
    ),
    (PipelineSource::Trigger, "trigger"),
    (PipelineSource::Web, "web"),
    (PipelineSource::WebIde, "web_ide"),
];

const PIPELINE_STATUS_TABLE: &[(PipelineStatus, &str)] = &[
    (PipelineStatus::Created, "created"),
    (PipelineStatus::WaitingForResource, "waiting_for_resource"),
    (PipelineStatus::Preparing, "preparing"),
    (PipelineStatus::Pending, "pending"),
    (PipelineStatus::Running, "running"),
    (PipelineStatus::Success, "success"),
    (PipelineStatus::Failed, "failed"),
    (PipelineStatus::Canceled, "canceled"),
    (PipelineStatus::Skipped, "skipped"),
    (PipelineStatus::Manual, "manual"),
    (PipelineStatus::Scheduled, "scheduled"),
    (PipelineStatus::Completed, "completed"),
    (PipelineStatus::Neutral, "neutral"),
    (PipelineStatus::Stale, "stale"),
    (PipelineStatus::StartupFailure, "startup_failure"),
    (PipelineStatus::TimedOut, "timed_out"),
];

impl JsonConvert<Pipeline<VecLookup>> for PipelineJson {
    fn convert_to_json(o: &Pipeline<VecLookup>) -> Self {
        Self {
            name: o.name.clone(),
            project: o.project.idx,
            sha: o.sha.clone(),
            previous_sha: o.previous_sha.clone(),
            refname: o.refname.clone(),
            stable_refname: o.stable_refname.clone(),
            source: enum_to_string(PIPELINE_SOURCE_TABLE, o.source).into(),
            schedule: o.schedule.map(|s| s.idx),
            parent_pipeline: o.parent_pipeline.map(|p| p.idx),
            merge_request: o.merge_request.map(|m| m.idx),
            variables: PipelineVariablesJson::convert_to_json(&o.variables),
            user: o.user.map(|u| u.idx),
            status: enum_to_string(PIPELINE_STATUS_TABLE, o.status).into(),
            coverage: o.coverage,
            forge_id: o.forge_id,
            url: o.url.clone(),
            archived: o.archived,
            created_at: o.created_at,
            updated_at: o.updated_at,
            started_at: o.started_at,
            finished_at: o.finished_at,
            cim_fetched_at: o.cim_fetched_at,
            cim_refreshed_at: o.cim_refreshed_at,
        }
    }

    fn create_from_json(&self) -> Result<Pipeline<VecLookup>, VecStoreError> {
        let mut pipeline = Pipeline::builder()
            .project(VecIndex::new(self.project))
            .sha(&self.sha)
            .source(enum_from_string(PIPELINE_SOURCE_TABLE, &self.source)?)
            .status(enum_from_string(PIPELINE_STATUS_TABLE, &self.status)?)
            .forge_id(self.forge_id)
            .url(&self.url)
            .created_at(self.created_at)
            .updated_at(self.updated_at)
            .build()
            .unwrap();
        pipeline.name.clone_from(&self.name);
        pipeline.previous_sha.clone_from(&self.previous_sha);
        pipeline.refname.clone_from(&self.refname);
        pipeline.stable_refname.clone_from(&self.stable_refname);
        pipeline.schedule = self.schedule.map(VecIndex::new);
        pipeline.parent_pipeline = self.parent_pipeline.map(VecIndex::new);
        pipeline.merge_request = self.merge_request.map(VecIndex::new);
        pipeline.variables = self.variables.create_from_json()?;
        pipeline.user = self.user.map(VecIndex::new);
        pipeline.coverage = self.coverage;
        pipeline.archived = self.archived;
        pipeline.started_at = self.started_at;
        pipeline.finished_at = self.finished_at;
        pipeline.cim_fetched_at = self.cim_fetched_at;
        pipeline.cim_refreshed_at = self.cim_refreshed_at;

        Ok(pipeline)
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct PipelineScheduleJson {
    name: String,
    project: usize,
    ref_: String,
    variables: PipelineVariablesJson,
    forge_id: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    owner: usize,
    active: bool,
    next_run: Option<DateTime<Utc>>,
    cim_fetched_at: DateTime<Utc>,
    cim_refreshed_at: DateTime<Utc>,
}

impl JsonConvert<PipelineSchedule<VecLookup>> for PipelineScheduleJson {
    fn convert_to_json(o: &PipelineSchedule<VecLookup>) -> Self {
        Self {
            name: o.name.clone(),
            project: o.project.idx,
            ref_: o.ref_.clone(),
            variables: PipelineVariablesJson::convert_to_json(&o.variables),
            forge_id: o.forge_id,
            created_at: o.created_at,
            updated_at: o.updated_at,
            owner: o.owner.idx,
            active: o.active,
            next_run: o.next_run,
            cim_fetched_at: o.cim_fetched_at,
            cim_refreshed_at: o.cim_refreshed_at,
        }
    }

    fn create_from_json(&self) -> Result<PipelineSchedule<VecLookup>, VecStoreError> {
        let mut pipeline_schedule = PipelineSchedule::builder()
            .project(VecIndex::new(self.project))
            .ref_(&self.ref_)
            .forge_id(self.forge_id)
            .created_at(self.created_at)
            .updated_at(self.updated_at)
            .owner(VecIndex::new(self.owner))
            .build()
            .unwrap();
        pipeline_schedule.name.clone_from(&self.name);
        pipeline_schedule.variables = self.variables.create_from_json()?;
        pipeline_schedule.active = self.active;
        pipeline_schedule.next_run = self.next_run;
        pipeline_schedule.cim_fetched_at = self.cim_fetched_at;
        pipeline_schedule.cim_refreshed_at = self.cim_refreshed_at;

        Ok(pipeline_schedule)
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct ProjectJson {
    name: String,
    forge_id: u64,
    url: String,
    instance: usize,
    instance_path: String,
    cim_fetched_at: DateTime<Utc>,
    cim_refreshed_at: DateTime<Utc>,
}

impl JsonConvert<Project<VecLookup>> for ProjectJson {
    fn convert_to_json(o: &Project<VecLookup>) -> Self {
        Self {
            name: o.name.clone(),
            forge_id: o.forge_id,
            url: o.url.clone(),
            instance: o.instance.idx,
            instance_path: o.instance_path.clone(),
            cim_fetched_at: o.cim_fetched_at,
            cim_refreshed_at: o.cim_refreshed_at,
        }
    }

    fn create_from_json(&self) -> Result<Project<VecLookup>, VecStoreError> {
        let mut project = Project::builder()
            .forge_id(self.forge_id)
            .instance(VecIndex::new(self.instance))
            .build()
            .unwrap();
        project.name.clone_from(&self.name);
        project.url.clone_from(&self.url);
        project.instance_path.clone_from(&self.instance_path);
        project.cim_fetched_at = self.cim_fetched_at;
        project.cim_refreshed_at = self.cim_refreshed_at;

        Ok(project)
    }
}
