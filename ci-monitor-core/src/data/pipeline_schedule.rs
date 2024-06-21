// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};

use crate::data::{Instance, PipelineVariables, Project, User};
use crate::Lookup;

/// A pipeline schedule.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PipelineSchedule<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    // Metadata.
    /// The name of the pipeline schedule.
    pub name: String,

    // Repository metadata.
    /// The project the schedule is associated with.
    pub project: <L as Lookup<Project<L>>>::Index,
    /// The ref the pipeline builds when it builds.
    pub ref_: String,

    // Execution metadata.
    /// Variables the schedule makes available to pipelines it starts.
    pub variables: PipelineVariables,

    // Forge metadata.
    /// The ID of the pipeline schedule.
    pub forge_id: u64,
    /// When the pipeline schedule was created.
    pub created_at: DateTime<Utc>,
    /// When the pipeline schedule was last updated.
    pub updated_at: DateTime<Utc>,
    /// The owner of the pipeline schedule.
    pub owner: <L as Lookup<User<L>>>::Index,
    /// Whether the pipeline schedule is active or not.
    pub active: bool,
    /// When the schedule will next create a pipeline.
    pub next_run: Option<DateTime<Utc>>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
