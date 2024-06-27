// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use perfect_derive::perfect_derive;

use crate::data::{Environment, Instance, MergeRequest, Pipeline, PipelineSchedule, Project, User};
use crate::Lookup;

/// The status of a deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeploymentStatus {
    /// The deployment has been created.
    Created,
    /// The deployment is running.
    Running,
    /// The deployment completed successfully.
    Success,
    /// The deployment completed with failure.
    Failed,
    /// The deployment was canceled.
    Canceled,
    /// The deployment is blocked.
    Blocked,
}

/// A deployment into an environment.
#[perfect_derive(Debug, Clone)]
#[non_exhaustive]
pub struct Deployment<L>
where
    L: Lookup<Environment<L>>,
    L: Lookup<Instance>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<Pipeline<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    // Project metadata.
    /// The pipeline which created the deployment.
    pub pipeline: <L as Lookup<Pipeline<L>>>::Index,
    /// The environment which was deployed into.
    pub environment: <L as Lookup<Environment<L>>>::Index,

    // Forge metadata.
    /// The ID of the deployment.
    pub forge_id: u64,
    /// When the deployment was created.
    pub created_at: DateTime<Utc>,
    /// When the deployment was updated.
    pub updated_at: DateTime<Utc>,
    /// When the deployment completed.
    pub finished_at: Option<DateTime<Utc>>,
    /// The status of the deployment.
    pub status: DeploymentStatus,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
