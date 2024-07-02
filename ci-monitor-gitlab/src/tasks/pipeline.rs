// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;

use chrono::{DateTime, Utc};
use ci_monitor_core::data::{
    Instance, MergeRequest, Pipeline, PipelineSchedule, PipelineSource, PipelineStatus, Project,
    User,
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
struct GitlabPipeline {
    id: u64,
    project_id: u64,
}

pub async fn discover_pipelines<L>(
    forge: &GitlabForge<L>,
    project: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_pipelines = {
        let endpoint = gitlab::api::projects::pipelines::Pipelines::builder()
            .project(project)
            .build()
            .unwrap();
        let endpoint = gitlab::api::paged(endpoint, gitlab::api::Pagination::All);
        endpoint.into_iter_async::<_, GitlabPipeline>(forge.gitlab())
    };

    let mut outcome = ForgeTaskOutcome::default();

    let tasks = gl_pipelines
        .map_ok(|pipeline| {
            ForgeTask::UpdatePipeline {
                project: pipeline.project_id,
                pipeline: pipeline.id,
            }
        })
        .map_err(errors::forge_error)
        .try_collect::<Vec<_>>()
        .await?;

    outcome.additional_tasks = tasks;

    Ok(outcome)
}

pub async fn discover_merge_request_pipelines<L>(
    forge: &GitlabForge<L>,
    project: u64,
    merge_request: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_pipelines = {
        let endpoint = gitlab::api::projects::merge_requests::MergeRequestPipelines::builder()
            .project(project)
            .merge_request(merge_request)
            .build()
            .unwrap();
        let endpoint = gitlab::api::paged(endpoint, gitlab::api::Pagination::All);
        endpoint.into_iter_async::<_, GitlabPipeline>(forge.gitlab())
    };

    let mut outcome = ForgeTaskOutcome::default();

    let tasks = gl_pipelines
        .map_ok(|pipeline| {
            ForgeTask::UpdatePipeline {
                project: pipeline.project_id,
                pipeline: pipeline.id,
            }
        })
        .map_err(errors::forge_error)
        .try_collect::<Vec<_>>()
        .await?;

    outcome.additional_tasks = tasks;

    Ok(outcome)
}

#[derive(Debug, Deserialize, Clone, Copy)]
enum GitlabPipelineSource {
    #[serde(rename = "push")]
    Push,
    #[serde(rename = "web")]
    Web,
    #[serde(rename = "trigger")]
    Trigger,
    #[serde(rename = "schedule")]
    Schedule,
    #[serde(rename = "api")]
    Api,
    #[serde(rename = "external")]
    External,
    #[serde(rename = "pipeline")]
    Pipeline,
    #[serde(rename = "chat")]
    Chat,
    #[serde(rename = "web_ide")]
    WebIde,
    #[serde(rename = "merge_request_event")]
    MergeRequestEvent,
    #[serde(rename = "external_pull_request_event")]
    ExternalPullRequestEvent,
    #[serde(rename = "parent_pipeline")]
    ParentPipeline,
    #[serde(rename = "ondemand_dast_scan")]
    OnDemandDastScan,
    #[serde(rename = "ondemand_dast_validation")]
    OnDemandDastValidation,
    #[serde(rename = "security_orchestration_policy")]
    SecurityOrchestrationPolicy,
}

impl From<GitlabPipelineSource> for PipelineSource {
    fn from(gps: GitlabPipelineSource) -> Self {
        match gps {
            GitlabPipelineSource::Push => Self::Push,
            GitlabPipelineSource::Web => Self::Web,
            GitlabPipelineSource::Trigger => Self::Trigger,
            GitlabPipelineSource::Schedule => Self::Schedule,
            GitlabPipelineSource::Api => Self::Api,
            GitlabPipelineSource::External => Self::External,
            GitlabPipelineSource::Pipeline => Self::Pipeline,
            GitlabPipelineSource::Chat => Self::Chat,
            GitlabPipelineSource::WebIde => Self::WebIde,
            GitlabPipelineSource::MergeRequestEvent => Self::MergeRequestEvent,
            GitlabPipelineSource::ExternalPullRequestEvent => Self::ExternalPullRequestEvent,
            GitlabPipelineSource::ParentPipeline => Self::ParentPipeline,
            GitlabPipelineSource::OnDemandDastScan => Self::OnDemandDastScan,
            GitlabPipelineSource::OnDemandDastValidation => Self::OnDemandDastValidation,
            GitlabPipelineSource::SecurityOrchestrationPolicy => Self::SecurityOrchestrationPolicy,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
enum GitlabPipelineStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "canceled")]
    Canceled,
    #[serde(rename = "skipped")]
    Skipped,
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "scheduled")]
    Scheduled,
    #[serde(rename = "preparing")]
    Preparing,
    #[serde(rename = "waiting_for_resource")]
    WaitingForResource,
}

impl From<GitlabPipelineStatus> for PipelineStatus {
    fn from(gps: GitlabPipelineStatus) -> Self {
        match gps {
            GitlabPipelineStatus::Running => Self::Running,
            GitlabPipelineStatus::Pending => Self::Pending,
            GitlabPipelineStatus::Success => Self::Success,
            GitlabPipelineStatus::Failed => Self::Failed,
            GitlabPipelineStatus::Canceled => Self::Canceled,
            GitlabPipelineStatus::Skipped => Self::Skipped,
            GitlabPipelineStatus::Created => Self::Created,
            GitlabPipelineStatus::Manual => Self::Manual,
            GitlabPipelineStatus::Scheduled => Self::Scheduled,
            GitlabPipelineStatus::Preparing => Self::Preparing,
            GitlabPipelineStatus::WaitingForResource => Self::WaitingForResource,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GitlabUser {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct GitlabPipelineDetails {
    id: u64,
    project_id: u64,

    name: Option<String>,
    sha: String,
    previous_sha: Option<String>,
    #[serde(rename = "ref")]
    ref_: Option<String>,
    source: GitlabPipelineSource,
    user: Option<GitlabUser>,
    status: GitlabPipelineStatus,
    coverage: Option<String>,
    web_url: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
}

pub async fn update_pipeline<L>(
    forge: &GitlabForge<L>,
    project: u64,
    pipeline: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<Pipeline<L>>,
    L: DiscoverableLookup<Project<L>>,
    L: DiscoverableLookup<User<L>>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_pipeline: GitlabPipelineDetails = {
        let endpoint = gitlab::api::projects::pipelines::Pipeline::builder()
            .project(project)
            .pipeline(pipeline)
            .build()
            .unwrap();
        endpoint
            .query_async(forge.gitlab())
            .await
            .map_err(errors::forge_error)?
    };

    let mut outcome = ForgeTaskOutcome::default();
    let mut add_task = |task| outcome.additional_tasks.push(task);
    let pipeline = gl_pipeline.id;

    let user_idx = if let Some(user) = gl_pipeline.user {
        if let Some(idx) =
            <L as DiscoverableLookup<User<L>>>::find(forge.storage().deref(), user.id)
        {
            Some(idx)
        } else {
            add_task(ForgeTask::UpdateUser {
                user: user.id,
            });
            None
        }
    } else {
        None
    };
    let project_idx = if let Some(idx) =
        <L as DiscoverableLookup<Project<L>>>::find(forge.storage().deref(), gl_pipeline.project_id)
    {
        Some(idx)
    } else {
        add_task(ForgeTask::UpdateProject {
            project: gl_pipeline.project_id,
        });
        None
    };

    let project_idx = if let Some(p) = project_idx {
        p
    } else {
        add_task(ForgeTask::UpdatePipeline {
            project,
            pipeline,
        });
        return Ok(outcome);
    };

    add_task(ForgeTask::DiscoverJobs {
        project: gl_pipeline.project_id,
        pipeline: gl_pipeline.id,
    });

    let update = move |pipeline: &mut Pipeline<L>| {
        pipeline.status = gl_pipeline.status.into();
        pipeline.coverage = gl_pipeline.coverage.and_then(|c| c.parse().ok());
        if user_idx.is_some() {
            pipeline.user = user_idx;
        }
        // TODO: How to tell if the pipeline is archived or not?
        //pipeline.archived = gl_pipeline.archived;
        pipeline.started_at = gl_pipeline.started_at;
        pipeline.finished_at = gl_pipeline.finished_at;

        pipeline.cim_refreshed_at = Utc::now();
    };

    // Create a pipeline entry.
    let pipeline = if let Some(idx) =
        <L as DiscoverableLookup<Pipeline<L>>>::find(forge.storage().deref(), pipeline)
    {
        if let Some(existing) = <L as Lookup<Pipeline<L>>>::lookup(forge.storage().deref(), &idx) {
            let mut updated = existing.clone();
            update(&mut updated);
            updated
        } else {
            return Err(ForgeError::lookup::<L, Pipeline<L>>(&idx));
        }
    } else {
        let mut pipeline = Pipeline::builder()
            .forge_id(pipeline)
            .project(project_idx)
            .sha(gl_pipeline.sha)
            .previous_sha(gl_pipeline.previous_sha)
            .refname(gl_pipeline.ref_.unwrap_or_else(|| "refs/UNKNOWN".into()))
            .stable_refname(Some(format!("refs/pipelines/{}", gl_pipeline.id)))
            .source(gl_pipeline.source.into())
            // TODO: How/where to obtain this information in this direction?
            //.schedule???
            //.parent_pipeline???
            //.merge_request???
            .status(gl_pipeline.status.into())
            .url(gl_pipeline.web_url)
            .created_at(gl_pipeline.created_at)
            .updated_at(gl_pipeline.updated_at)
            .name(gl_pipeline.name)
            .build()
            .unwrap();

        update(&mut pipeline);
        pipeline
    };

    // Store the pipeline in the storage.
    forge.storage_mut().store(pipeline);

    Ok(outcome)
}
