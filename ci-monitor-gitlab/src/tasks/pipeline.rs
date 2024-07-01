// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ci_monitor_core::data::Instance;
use ci_monitor_core::Lookup;
use ci_monitor_forge::{ForgeError, ForgeTask, ForgeTaskOutcome};
use futures_util::stream::TryStreamExt;
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
