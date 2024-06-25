// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::Utc;
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
    name: String,
    web_url: String,
    path_with_namespace: String,

    // Options which can discover more work.
    merge_requests_access_level: AccessLevel,
    builds_access_level: AccessLevel,
    environments_access_level: AccessLevel,
    forked_from_project: Option<ParentProject>,
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
    let mut outcome = ForgeTaskOutcome::default();
    let mut add_task = |task| outcome.additional_tasks.push(task);

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

    // TODO: update storage

    Ok(outcome)
}
