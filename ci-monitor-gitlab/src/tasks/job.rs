// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;

use chrono::{DateTime, Utc};
use ci_monitor_core::data::{
    Deployment, Environment, Instance, Job, JobState, MergeRequest, Pipeline, PipelineSchedule,
    Project, Runner, RunnerHost, User,
};
use ci_monitor_core::Lookup;
use ci_monitor_forge::{ForgeError, ForgeTask, ForgeTaskOutcome};
use ci_monitor_persistence::DiscoverableLookup;
use futures_util::stream::TryStreamExt;
use gitlab::api::AsyncQuery;
use serde::Deserialize;

use crate::errors;
use crate::GitlabForge;

#[derive(Debug, Deserialize)]
struct GitlabJob {
    id: u64,
}

pub async fn discover_jobs<L>(
    forge: &GitlabForge<L>,
    project: u64,
    pipeline: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_jobs = {
        let endpoint = gitlab::api::projects::pipelines::PipelineJobs::builder()
            .project(project)
            .pipeline(pipeline)
            .include_retried(true)
            .build()
            .unwrap();
        let endpoint = gitlab::api::paged(endpoint, gitlab::api::Pagination::All);
        endpoint.into_iter_async::<_, GitlabJob>(forge.gitlab())
    };

    let mut outcome = ForgeTaskOutcome::default();

    let tasks = gl_jobs
        .map_ok(|job| {
            ForgeTask::UpdateJob {
                project,
                job: job.id,
            }
        })
        .map_err(errors::forge_error)
        .try_collect::<Vec<_>>()
        .await?;

    outcome.additional_tasks = tasks;

    Ok(outcome)
}

#[derive(Debug, Deserialize, Clone, Copy)]
enum GitlabJobStatus {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "canceled")]
    Canceled,
    #[serde(rename = "skipped")]
    Skipped,
    #[serde(rename = "waiting_for_resource")]
    WaitingForResource,
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "scheduled")]
    Scheduled,
}

impl From<GitlabJobStatus> for JobState {
    fn from(gjs: GitlabJobStatus) -> Self {
        match gjs {
            GitlabJobStatus::Created => Self::Created,
            GitlabJobStatus::Pending => Self::Pending,
            GitlabJobStatus::Running => Self::Running,
            GitlabJobStatus::Failed => Self::Failed,
            GitlabJobStatus::Success => Self::Success,
            GitlabJobStatus::Canceled => Self::Canceled,
            GitlabJobStatus::Skipped => Self::Skipped,
            GitlabJobStatus::WaitingForResource => Self::WaitingForResource,
            GitlabJobStatus::Manual => Self::Manual,
            GitlabJobStatus::Scheduled => Self::Scheduled,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GitlabUser {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct GitlabPipeline {
    id: u64,
    project_id: u64,
}

#[derive(Debug, Deserialize)]
struct GitlabRunner {
    id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum GitlabCoverage {
    Float(f64),
    String(String),
}

impl GitlabCoverage {
    fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::String(s) => s.parse().ok(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GitlabJobDetails {
    id: u64,
    user: GitlabUser,

    name: String,
    stage: String,
    status: GitlabJobStatus,
    allow_failure: bool,
    tag_list: Vec<String>,
    web_url: String,
    pipeline: GitlabPipeline,
    runner: Option<GitlabRunner>,

    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    erased_at: Option<DateTime<Utc>>,
    queued_duration: Option<f64>,
    archived: bool,
    coverage: Option<GitlabCoverage>,
}

pub async fn update_job<L>(
    forge: &GitlabForge<L>,
    project: u64,
    job: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<Job<L>>,
    L: DiscoverableLookup<Pipeline<L>>,
    L: DiscoverableLookup<Runner<L>>,
    L: DiscoverableLookup<User<L>>,
    L: Lookup<Deployment<L>>,
    L: Lookup<Environment<L>>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Project<L>>,
    L: Lookup<RunnerHost>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_job: GitlabJobDetails = {
        let endpoint = gitlab::api::projects::jobs::Job::builder()
            .project(project)
            .job(job)
            .build()
            .unwrap();
        endpoint
            .query_async(forge.gitlab())
            .await
            .map_err(errors::forge_error)?
    };

    let mut outcome = ForgeTaskOutcome::default();
    let mut add_task = |task| outcome.additional_tasks.push(task);
    let job = gl_job.id;

    let user_idx = if let Some(idx) =
        <L as DiscoverableLookup<User<L>>>::find(forge.storage().deref(), gl_job.user.id)
    {
        Some(idx)
    } else {
        add_task(ForgeTask::UpdateUser {
            user: gl_job.user.id,
        });
        None
    };
    let pipeline_idx = if let Some(idx) =
        <L as DiscoverableLookup<Pipeline<L>>>::find(forge.storage().deref(), gl_job.pipeline.id)
    {
        Some(idx)
    } else {
        add_task(ForgeTask::UpdatePipeline {
            project: gl_job.pipeline.project_id,
            pipeline: gl_job.pipeline.id,
        });
        None
    };
    let runner_idx = if let Some(runner) = gl_job.runner {
        if let Some(idx) =
            <L as DiscoverableLookup<Runner<L>>>::find(forge.storage().deref(), runner.id)
        {
            Some(idx)
        } else {
            add_task(ForgeTask::UpdateRunner {
                id: runner.id,
            });
            None
        }
    } else {
        None
    };

    let (user_idx, pipeline_idx) =
        if let Some((u, p)) = user_idx.and_then(|u| pipeline_idx.map(|p| (u, p))) {
            (u, p)
        } else {
            add_task(ForgeTask::UpdateJob {
                project,
                job,
            });
            return Ok(outcome);
        };

    let update = move |job: &mut Job<L>| {
        job.state = gl_job.status.into();
        job.started_at = gl_job.started_at;
        job.finished_at = gl_job.finished_at;
        job.erased_at = gl_job.erased_at;
        job.queued_duration = gl_job.queued_duration;
        job.archived = gl_job.archived;
        job.coverage = gl_job.coverage.and_then(|c| c.as_f64());

        job.cim_refreshed_at = Utc::now();
    };

    // Create a job entry.
    let job =
        if let Some(idx) = <L as DiscoverableLookup<Job<L>>>::find(forge.storage().deref(), job) {
            if let Some(existing) = <L as Lookup<Job<L>>>::lookup(forge.storage().deref(), &idx) {
                let mut updated = existing.clone();
                update(&mut updated);
                updated
            } else {
                return Err(ForgeError::lookup::<L, Job<L>>(&idx));
            }
        } else {
            let mut job = Job::builder()
                .user(user_idx)
                .state(gl_job.status.into())
                .created_at(gl_job.created_at)
                .runner(runner_idx)
                .forge_id(job)
                .pipeline(pipeline_idx)
                .name(gl_job.name)
                .stage(gl_job.stage)
                .allow_failure(gl_job.allow_failure)
                .tags(gl_job.tag_list)
                //.variables(gl_job.variables)
                //.deployment
                .url(gl_job.web_url)
                .build()
                .unwrap();

            update(&mut job);
            job
        };

    // Store the job in the storage.
    forge.storage_mut().store(job);

    Ok(outcome)
}
