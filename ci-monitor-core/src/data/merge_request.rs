// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};

use crate::data::{Instance, Project, User};
use crate::Lookup;

/// The status of a merge request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MergeRequestStatus {
    /// The merge request is open.
    Open,
    /// The merge request has been closed without merging.
    Closed,
    /// The merge request has been merged.
    Merged,
}

/// A merge request.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MergeRequest<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    // Repository metadata.
    /// The user-visible ID of the merge request.
    pub id: u64,
    /// The source project.
    pub source_project: <L as Lookup<Project<L>>>::Index,
    /// The source branch.
    pub source_branch: String,
    /// The `HEAD` commit of the merge request.
    pub sha: String,
    /// The target project.
    pub target_project: <L as Lookup<Project<L>>>::Index,
    /// The target branch.
    pub target_branch: String,

    // Forge metadata.
    /// The id of the merge request.
    pub forge_id: u64,
    /// The title of the merge request.
    pub title: String,
    /// The description of the merge request.
    pub description: String,
    /// The state of the merge request.
    pub state: MergeRequestStatus,
    /// The author of the merge request.
    pub author: <L as Lookup<User<L>>>::Index,
    /// The URL of the pipeline webpage.
    pub url: String,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
