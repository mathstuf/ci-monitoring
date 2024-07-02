// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::Deref;

use chrono::{DateTime, Utc};
use ci_monitor_core::data::{
    Instance, Project, Runner, RunnerHost, RunnerProtectionLevel, RunnerType,
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
struct GitlabRunner {
    id: u64,
}

pub async fn discover_runners<L>(forge: &GitlabForge<L>) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_runners = {
        let endpoint = gitlab::api::runners::AllRunners::builder().build().unwrap();
        let endpoint = gitlab::api::paged(endpoint, gitlab::api::Pagination::All);
        endpoint.into_iter_async::<_, GitlabRunner>(forge.gitlab())
    };

    let mut outcome = ForgeTaskOutcome::default();

    let tasks = gl_runners
        .map_ok(|runner| {
            ForgeTask::UpdateRunner {
                id: runner.id,
            }
        })
        .map_err(errors::forge_error)
        .try_collect::<Vec<_>>()
        .await?;

    outcome.additional_tasks = tasks;

    Ok(outcome)
}

#[derive(Debug, Deserialize, Clone, Copy)]
enum GitlabRunnerType {
    #[serde(rename = "instance_type")]
    Instance,
    #[serde(rename = "group_type")]
    Group,
    #[serde(rename = "project_type")]
    Project,
}

impl From<GitlabRunnerType> for RunnerType {
    fn from(grt: GitlabRunnerType) -> Self {
        match grt {
            GitlabRunnerType::Instance => Self::Instance,
            GitlabRunnerType::Group => Self::Group,
            GitlabRunnerType::Project => Self::Project,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum GitlabRunnerAccessLevel {
    #[serde(rename = "ref_protected")]
    RefProtected,
    #[serde(rename = "not_protected")]
    NotProtected,
}

impl From<GitlabRunnerAccessLevel> for RunnerProtectionLevel {
    fn from(gral: GitlabRunnerAccessLevel) -> Self {
        match gral {
            GitlabRunnerAccessLevel::RefProtected => Self::Protected,
            GitlabRunnerAccessLevel::NotProtected => Self::Any,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GitlabRunnerDetails {
    id: u64,
    description: String,
    runner_type: GitlabRunnerType,

    name: Option<String>,
    version: Option<String>,
    revision: Option<String>,
    platform: Option<String>,
    architecture: Option<String>,

    tag_list: Vec<String>,
    run_untagged: bool,
    access_level: GitlabRunnerAccessLevel,

    maintenance_note: Option<String>,
    contacted_at: Option<DateTime<Utc>>,

    paused: bool,
    is_shared: bool,
    online: Option<bool>,
    locked: bool,

    maximum_timeout: Option<u64>,
}

pub async fn update_runner<L>(
    forge: &GitlabForge<L>,
    runner: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<Project<L>>,
    L: DiscoverableLookup<Runner<L>>,
    L: Lookup<RunnerHost>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let gl_runner: GitlabRunnerDetails = {
        let endpoint = gitlab::api::runners::Runner::builder()
            .runner(runner)
            .build()
            .unwrap();
        endpoint
            .query_async(forge.gitlab())
            .await
            .map_err(errors::forge_error)?
    };

    let outcome = ForgeTaskOutcome::default();
    let runner = gl_runner.id;

    let update = move |runner: &mut Runner<L>| {
        runner.description = gl_runner.description;
        runner.maximum_timeout = gl_runner.maximum_timeout;
        runner.protection_level = gl_runner.access_level.into();
        runner.implementation = gl_runner.name.unwrap_or_default();
        runner.version = gl_runner.version.unwrap_or_default();
        runner.revision = gl_runner.revision.unwrap_or_default();
        runner.platform = gl_runner.platform.unwrap_or_default();
        runner.architecture = gl_runner.architecture.unwrap_or_default();
        runner.tags = gl_runner.tag_list;
        runner.run_untagged = gl_runner.run_untagged;
        // TODO: Get the list of projects, for each:
        // - if it exists, get the index
        // - if not, queue an update for the project and this runner
        //runner.projects = gl_runner.projects;
        runner.paused = gl_runner.paused;
        runner.shared = gl_runner.is_shared;
        runner.online = gl_runner.online.unwrap_or(false);
        runner.locked = gl_runner.locked;
        runner.contacted_at = gl_runner.contacted_at;
        runner.maintenance_note = gl_runner.maintenance_note;

        runner.cim_refreshed_at = Utc::now();
    };

    // Create a runner entry.
    let runner_entry = if let Some(idx) =
        <L as DiscoverableLookup<Runner<L>>>::find(forge.storage().deref(), runner)
    {
        if let Some(existing) = <L as Lookup<Runner<L>>>::lookup(forge.storage().deref(), &idx) {
            let mut updated = existing.clone();
            update(&mut updated);
            updated
        } else {
            return Err(ForgeError::lookup::<L, Runner<L>>(&idx));
        }
    } else {
        let mut runner = Runner::builder()
            .forge_id(runner)
            .instance(forge.instance_index())
            .runner_type(gl_runner.runner_type.into())
            .protection_level(gl_runner.access_level.into())
            .build()
            .unwrap();

        update(&mut runner);
        runner
    };

    // Store the runner in the storage.
    forge.storage_mut().store(runner_entry);

    Ok(outcome)
}
