// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_trait::async_trait;
use ci_monitor_core::data::Instance;
use ci_monitor_core::Lookup;
use ci_monitor_forge::{Forge, ForgeCore, ForgeError, ForgeTask, ForgeTaskOutcome};
use gitlab::AsyncGitlab;

use crate::tasks;
use crate::GitlabLookup;

/// A CI monitoring task handler for GitLab hosts.
pub struct GitlabForge<L>
where
    L: Lookup<Instance>,
{
    url: String,
    gitlab: AsyncGitlab,
    storage: RwLock<L>,
    instance_idx: <L as Lookup<Instance>>::Index,
}

impl<L> GitlabForge<L>
where
    L: Lookup<Instance>,
{
    pub(crate) fn gitlab(&self) -> &AsyncGitlab {
        &self.gitlab
    }

    pub(crate) fn storage(&self) -> RwLockReadGuard<L> {
        self.storage.read().unwrap()
    }

    pub(crate) fn storage_mut(&self) -> RwLockWriteGuard<L> {
        self.storage.write().unwrap()
    }

    pub(crate) fn instance_index(&self) -> <L as Lookup<Instance>>::Index {
        self.instance_idx.clone()
    }
}

impl<L> ForgeCore for GitlabForge<L>
where
    L: Lookup<Instance>,
{
    fn instance(&self) -> Instance {
        Instance::builder()
            .forge("gitlab")
            .url(self.url.clone())
            .build()
            .unwrap()
    }
}

#[async_trait]
impl<L> Forge for GitlabForge<L>
where
    L: GitlabLookup<L> + Clone + Send + Sync,
{
    /// Run a task.
    async fn run_task_async(&self, task: ForgeTask) -> Result<ForgeTaskOutcome, ForgeError> {
        match task {
            ForgeTask::UpdateProject {
                project,
            } => tasks::update_project(self, project).await,
            ForgeTask::UpdateProjectByName {
                project,
            } => tasks::update_project_by_name(self, project).await,
            ForgeTask::UpdateUserByName {
                user,
            } => tasks::update_user_by_name(self, user).await,
            ForgeTask::UpdateUser {
                user,
            } => tasks::update_user(self, user).await,
            ForgeTask::DiscoverRunners => tasks::discover_runners(self).await,
            ForgeTask::UpdateRunner {
                id,
            } => tasks::update_runner(self, id).await,
            ForgeTask::DiscoverPipelineSchedules {
                project,
            } => tasks::discover_pipeline_schedules(self, project).await,
            ForgeTask::UpdatePipelineSchedule {
                project,
                schedule,
            } => tasks::update_pipeline_schedule(self, project, schedule).await,
            ForgeTask::DiscoverMergeRequests {
                project,
            } => tasks::discover_merge_requests(self, project).await,
            ForgeTask::UpdateMergeRequest {
                project,
                merge_request,
            } => tasks::update_merge_request(self, project, merge_request).await,
            ForgeTask::DiscoverPipelines {
                project,
            } => tasks::discover_pipelines(self, project).await,
            ForgeTask::UpdatePipeline {
                project,
                pipeline,
            } => tasks::update_pipeline(self, project, pipeline).await,
            _ => {
                Err(ForgeError::Unknown {
                    task,
                })
            },
        }
    }
}
