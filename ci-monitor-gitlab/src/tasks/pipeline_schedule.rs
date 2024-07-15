// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;

use chrono::{DateTime, Utc};
use ci_monitor_core::data::{Instance, PipelineSchedule, Project, User};
use ci_monitor_core::Lookup;
use ci_monitor_forge::{ForgeError, ForgeTask, ForgeTaskOutcome};
use ci_monitor_persistence::DiscoverableLookup;
use futures_util::stream::TryStreamExt;
use gitlab::api::AsyncQuery;
use serde::Deserialize;

use crate::errors;
use crate::tasks::GitlabPipelineVariable;
use crate::GitlabForge;

#[derive(Debug, Deserialize)]
struct GitlabPipelineSchedule {
    id: u64,
}

pub async fn discover_pipeline_schedules<L>(
    forge: &GitlabForge<L>,
    project: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_pipeline_schedules = {
        let endpoint = gitlab::api::projects::pipeline_schedules::PipelineSchedules::builder()
            .project(project)
            .build()
            .unwrap();
        let endpoint = gitlab::api::paged(endpoint, gitlab::api::Pagination::All);
        endpoint.into_iter_async::<_, GitlabPipelineSchedule>(forge.gitlab())
    };

    let mut outcome = ForgeTaskOutcome::default();

    let tasks = gl_pipeline_schedules
        .map_ok(|pipeline_schedule| {
            ForgeTask::UpdatePipelineSchedule {
                project,
                schedule: pipeline_schedule.id,
            }
        })
        .map_err(errors::forge_error)
        .try_collect::<Vec<_>>()
        .await?;

    outcome.additional_tasks = tasks;

    Ok(outcome)
}

#[derive(Debug, Deserialize)]
struct GitlabUser {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct GitlabPipelineScheduleDetails {
    id: u64,
    description: String,

    #[serde(rename = "ref")]
    ref_: String,

    #[serde(default)]
    variables: Vec<GitlabPipelineVariable>,

    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    owner: GitlabUser,

    active: bool,
    next_run_at: Option<DateTime<Utc>>,
}

pub async fn update_pipeline_schedule<L>(
    forge: &GitlabForge<L>,
    project: u64,
    pipeline_schedule: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<PipelineSchedule<L>>,
    L: DiscoverableLookup<Project<L>>,
    L: DiscoverableLookup<User<L>>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_pipeline_schedule: GitlabPipelineScheduleDetails = {
        let endpoint = gitlab::api::projects::pipeline_schedules::PipelineSchedule::builder()
            .project(project)
            .id(pipeline_schedule)
            .build()
            .unwrap();
        endpoint
            .query_async(forge.gitlab())
            .await
            .map_err(errors::forge_error)?
    };

    let mut outcome = ForgeTaskOutcome::default();
    let mut add_task = |task| outcome.additional_tasks.push(task);
    let pipeline_schedule = gl_pipeline_schedule.id;

    let user_idx = if let Some(idx) = <L as DiscoverableLookup<User<L>>>::find(
        forge.storage().deref(),
        gl_pipeline_schedule.owner.id,
    ) {
        Some(idx)
    } else {
        add_task(ForgeTask::UpdateUser {
            user: gl_pipeline_schedule.owner.id,
        });
        None
    };
    let project_idx = if let Some(idx) =
        <L as DiscoverableLookup<Project<L>>>::find(forge.storage().deref(), project)
    {
        Some(idx)
    } else {
        add_task(ForgeTask::UpdateProject {
            project,
        });
        None
    };

    let (user_idx, project_idx) =
        if let Some((u, p)) = user_idx.and_then(|u| project_idx.map(|p| (u, p))) {
            (u, p)
        } else {
            add_task(ForgeTask::UpdatePipelineSchedule {
                project,
                schedule: pipeline_schedule,
            });
            return Ok(outcome);
        };
    let user_idx_inner = user_idx.clone();
    let ref_inner = gl_pipeline_schedule.ref_.clone();

    let update = move |pipeline_schedule: &mut PipelineSchedule<L>| {
        pipeline_schedule.name = gl_pipeline_schedule.description;
        pipeline_schedule.ref_ = ref_inner;
        pipeline_schedule.updated_at = gl_pipeline_schedule.updated_at;
        pipeline_schedule.active = gl_pipeline_schedule.active;
        pipeline_schedule.next_run = gl_pipeline_schedule.next_run_at;
        pipeline_schedule.owner = user_idx_inner;
        pipeline_schedule.variables = super::gitlab_variables(gl_pipeline_schedule.variables);

        pipeline_schedule.cim_refreshed_at = Utc::now();
    };

    // Create a pipeline schedule entry.
    let pipeline_schedule = if let Some(idx) = <L as DiscoverableLookup<PipelineSchedule<L>>>::find(
        forge.storage().deref(),
        pipeline_schedule,
    ) {
        if let Some(existing) =
            <L as Lookup<PipelineSchedule<L>>>::lookup(forge.storage().deref(), &idx)
        {
            let mut updated = existing.clone();
            update(&mut updated);
            updated
        } else {
            return Err(ForgeError::lookup::<L, PipelineSchedule<L>>(&idx));
        }
    } else {
        let mut pipeline_schedule = PipelineSchedule::builder()
            .forge_id(pipeline_schedule)
            .project(project_idx)
            .ref_(gl_pipeline_schedule.ref_)
            .created_at(gl_pipeline_schedule.created_at)
            .updated_at(gl_pipeline_schedule.updated_at)
            .owner(user_idx)
            .build()
            .unwrap();

        update(&mut pipeline_schedule);
        pipeline_schedule
    };

    // Store the pipeline schedule in the storage.
    forge.storage_mut().store(pipeline_schedule);

    Ok(outcome)
}
