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
