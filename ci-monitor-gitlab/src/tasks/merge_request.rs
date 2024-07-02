// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;

use chrono::Utc;
use ci_monitor_core::data::{
    Instance, MergeRequest, MergeRequestStatus, PipelineSchedule, Project, User,
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
struct GitlabMergeRequest {
    iid: u64,
}

pub async fn discover_merge_requests<L>(
    forge: &GitlabForge<L>,
    project: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_merge_requests = {
        let endpoint = gitlab::api::projects::merge_requests::MergeRequests::builder()
            .project(project)
            .build()
            .unwrap();
        let endpoint = gitlab::api::paged(endpoint, gitlab::api::Pagination::All);
        endpoint.into_iter_async::<_, GitlabMergeRequest>(forge.gitlab())
    };

    let mut outcome = ForgeTaskOutcome::default();

    let tasks = gl_merge_requests
        .map_ok(|merge_request| {
            ForgeTask::UpdateMergeRequest {
                project,
                merge_request: merge_request.iid,
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

#[derive(Debug, Deserialize, Clone, Copy)]
enum GitlabMergeState {
    #[serde(rename = "opened")]
    Opened,
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "reopened")]
    Reopened,
    #[serde(rename = "merged")]
    Merged,
    #[serde(rename = "locked")]
    Locked,
}

impl From<GitlabMergeState> for MergeRequestStatus {
    fn from(gms: GitlabMergeState) -> Self {
        match gms {
            GitlabMergeState::Opened | GitlabMergeState::Reopened | GitlabMergeState::Locked => {
                Self::Open
            },
            GitlabMergeState::Closed => Self::Closed,
            GitlabMergeState::Merged => Self::Merged,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GitlabMergeRequestDetails {
    id: u64,
    iid: u64,

    author: GitlabUser,
    web_url: String,

    title: String,
    description: String,

    state: GitlabMergeState,

    source_project_id: Option<u64>,
    source_branch: String,
    sha: Option<String>,
    target_project_id: u64,
    target_branch: String,
}

pub async fn update_merge_request<L>(
    forge: &GitlabForge<L>,
    project: u64,
    merge_request: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<MergeRequest<L>>,
    L: DiscoverableLookup<Project<L>>,
    L: DiscoverableLookup<User<L>>,
    L: Lookup<Instance>,
    L: Lookup<PipelineSchedule<L>>,
    L: Send + Sync,
{
    let gl_merge_request: GitlabMergeRequestDetails = {
        let endpoint = gitlab::api::projects::merge_requests::MergeRequest::builder()
            .project(project)
            .merge_request(merge_request)
            .build()
            .unwrap();
        endpoint
            .query_async(forge.gitlab())
            .await
            .map_err(errors::forge_error)?
    };

    let mut outcome = ForgeTaskOutcome::default();
    let mut add_task = |task| outcome.additional_tasks.push(task);
    let merge_request = gl_merge_request.id;

    let author_idx = if let Some(idx) = <L as DiscoverableLookup<User<L>>>::find(
        forge.storage().deref(),
        gl_merge_request.author.id,
    ) {
        Some(idx)
    } else {
        add_task(ForgeTask::UpdateUser {
            user: gl_merge_request.author.id,
        });
        None
    };
    let target_project_idx = if let Some(idx) = <L as DiscoverableLookup<Project<L>>>::find(
        forge.storage().deref(),
        gl_merge_request.target_project_id,
    ) {
        Some(idx)
    } else {
        add_task(ForgeTask::UpdateProject {
            project: gl_merge_request.target_project_id,
        });
        None
    };
    let source_project_idx = if let Some(source_project_id) = gl_merge_request.source_project_id {
        if source_project_id == gl_merge_request.target_project_id {
            target_project_idx.clone()
        } else if let Some(idx) =
            <L as DiscoverableLookup<Project<L>>>::find(forge.storage().deref(), source_project_id)
        {
            Some(idx)
        } else {
            add_task(ForgeTask::UpdateProject {
                project: source_project_id,
            });
            None
        }
    } else {
        // Just act as if the MR came from the target project itself.
        target_project_idx.clone()
    };

    let (author_idx, target_project_idx, source_project_idx) = if let Some((a, t, s)) = author_idx
        .and_then(|a| target_project_idx.and_then(|t| source_project_idx.map(|s| (a, t, s))))
    {
        (a, t, s)
    } else {
        add_task(ForgeTask::UpdateMergeRequest {
            project,
            merge_request,
        });
        return Ok(outcome);
    };

    add_task(ForgeTask::DiscoverMergeRequestPipelines {
        project,
        merge_request: gl_merge_request.iid,
    });

    let update = move |merge_request: &mut MergeRequest<L>| {
        merge_request.source_branch = gl_merge_request.source_branch;
        merge_request.sha = gl_merge_request.sha.unwrap_or_default();
        merge_request.target_branch = gl_merge_request.target_branch;
        merge_request.title = gl_merge_request.title;
        merge_request.description = gl_merge_request.description;
        merge_request.state = gl_merge_request.state.into();

        merge_request.cim_refreshed_at = Utc::now();
    };

    // Create a merge request entry.
    let merge_request = if let Some(idx) =
        <L as DiscoverableLookup<MergeRequest<L>>>::find(forge.storage().deref(), merge_request)
    {
        if let Some(existing) =
            <L as Lookup<MergeRequest<L>>>::lookup(forge.storage().deref(), &idx)
        {
            let mut updated = existing.clone();
            update(&mut updated);
            updated
        } else {
            return Err(ForgeError::lookup::<L, MergeRequest<L>>(&idx));
        }
    } else {
        let mut merge_request = MergeRequest::builder()
            .id(gl_merge_request.iid)
            .source_project(source_project_idx)
            .target_project(target_project_idx)
            .forge_id(gl_merge_request.id)
            .state(gl_merge_request.state.into())
            .author(author_idx)
            .url(gl_merge_request.web_url)
            .build()
            .unwrap();

        update(&mut merge_request);
        merge_request
    };

    // Store the merge request in the storage.
    forge.storage_mut().store(merge_request);

    Ok(outcome)
}
