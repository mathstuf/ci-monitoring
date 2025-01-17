// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;

use chrono::{DateTime, Utc};
use ci_monitor_core::data::{Instance, Project};
use ci_monitor_core::Lookup;
use ci_monitor_forge::{ForgeError, ForgeTask, ForgeTaskOutcome};
use ci_monitor_persistence::DiscoverableLookup;
use gitlab::api::AsyncQuery;
use serde::Deserialize;

use crate::errors;
use crate::GitlabForge;

#[derive(Debug, Deserialize, Clone, Copy)]
enum AccessLevel {
    #[serde(rename = "enabled")]
    Enabled,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "disabled")]
    Disabled,
}

impl AccessLevel {
    fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled | Self::Private)
    }
}

#[derive(Debug, Deserialize)]
struct ParentProject {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct GitlabProject {
    // Data to fill in the storage.
    id: u64,
    name: String,
    web_url: String,
    path_with_namespace: String,

    // Options which can discover more work.
    merge_requests_access_level: AccessLevel,
    builds_access_level: AccessLevel,
    environments_access_level: AccessLevel,
    forked_from_project: Option<ParentProject>,

    updated_at: DateTime<Utc>,
}

async fn update_project_impl<L>(
    forge: &GitlabForge<L>,
    gl_project: GitlabProject,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<Project<L>>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let mut outcome = ForgeTaskOutcome::default();
    let mut add_task = |task| outcome.additional_tasks.push(task);
    let project = gl_project.id;

    let update = move |project: &mut Project<L>| {
        project.name = gl_project.name;
        project.url = gl_project.web_url;
        project.instance_path = gl_project.path_with_namespace;

        project.cim_refreshed_at = Utc::now();
    };

    // Create a project entry.
    let (project_entry, update_components) = if let Some(idx) = forge.storage().find(project) {
        if let Some(existing) = <L as Lookup<Project<L>>>::lookup(forge.storage().deref(), &idx) {
            let mut updated = existing.clone();
            update(&mut updated);
            (updated, existing.cim_refreshed_at < gl_project.updated_at)
        } else {
            return Err(ForgeError::lookup::<L, Project<L>>(&idx));
        }
    } else {
        let mut project = Project::builder()
            .forge_id(project)
            .instance(forge.instance_index())
            .build()
            .unwrap();

        update(&mut project);
        (project, true)
    };

    if update_components {
        if gl_project.merge_requests_access_level.is_enabled() {
            add_task(ForgeTask::DiscoverMergeRequests {
                project,
            });
        }

        if gl_project.builds_access_level.is_enabled() {
            add_task(ForgeTask::DiscoverPipelineSchedules {
                project,
            });
            add_task(ForgeTask::DiscoverPipelines {
                project,
            });
        }

        if gl_project.environments_access_level.is_enabled() {
            add_task(ForgeTask::DiscoverEnvironments {
                project,
            });
            add_task(ForgeTask::DiscoverDeployments {
                project,
            });
        }

        if let Some(parent) = gl_project.forked_from_project {
            add_task(ForgeTask::UpdateProject {
                project: parent.id,
            })
        }
    }

    // Store the project in the storage.
    forge.storage_mut().store(project_entry);

    Ok(outcome)
}

pub async fn update_project<L>(
    forge: &GitlabForge<L>,
    project: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<Project<L>>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_project: GitlabProject = {
        let endpoint = gitlab::api::projects::Project::builder()
            .project(project)
            .build()
            .unwrap();
        endpoint
            .query_async(forge.gitlab())
            .await
            .map_err(errors::forge_error)?
    };

    update_project_impl(forge, gl_project).await
}

pub async fn update_project_by_name<L>(
    forge: &GitlabForge<L>,
    project: String,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<Project<L>>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_project: GitlabProject = {
        let endpoint = gitlab::api::projects::Project::builder()
            .project(project)
            .build()
            .unwrap();
        endpoint
            .query_async(forge.gitlab())
            .await
            .map_err(errors::forge_error)?
    };

    update_project_impl(forge, gl_project).await
}
