// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};

use crate::data::{Instance, Project, RunnerHost};
use crate::Lookup;

/// The scope at which a runner is registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RunnerType {
    /// Can accept instance-wide jobs.
    Instance,
    /// Can accept jobs from a specific group.
    Group,
    /// Can accept jobs from a specific project.
    Project,
}

/// Types of refs the runner may run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RunnerProtectionLevel {
    /// Only jobs for protected refs may use this runner.
    Protected,
    /// Any job can use this runner.
    Any,
}

/// A runner which can perform jobs for CI tasks.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Runner<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
    L: Lookup<RunnerHost>,
{
    // Metadata.
    /// The description of the runner.
    pub description: String,
    /// The runner type.
    pub runner_type: RunnerType,
    /// The maximum timeout for jobs on the runner (in seconds).
    pub maximum_timeout: Option<u64>,
    /// Protection level of refs that can use this runner.
    pub protection_level: RunnerProtectionLevel,

    // Runner program metadata.
    /// The implementation of the runner.
    pub implementation: String,
    /// The version of the runner.
    pub version: String,
    /// The revision of the runner.
    pub revision: String,
    /// The platform of the runner.
    pub platform: String,
    /// The CPU architecture of the runner.
    pub architecture: String,

    // Scheduling metadata.
    /// The tags for the runner.
    pub tags: Vec<String>,
    /// Whether untagged jobs may use this runner.
    pub run_untagged: bool,
    /// The set of projects which may use this runner.
    pub projects: Vec<<L as Lookup<Project<L>>>::Index>,

    // Forge metadata.
    /// The id of the runner.
    pub forge_id: u64,
    /// Whether the runner is paused or not.
    pub paused: bool,
    /// Whether the runner is shared with other projects or not.
    pub shared: bool,
    /// Whether the runner is online or not.
    pub online: bool,
    /// Whether the runner is locked to its projects or not.
    pub locked: bool,
    /// When the runner last contacted the forge.
    pub contacted_at: Option<DateTime<Utc>>,
    /// The maintenance note of the runner.
    pub maintenance_note: Option<String>,
    /// The instance for which the runner performs jobs.
    pub instance: <L as Lookup<Instance>>::Index,

    // Maintenance metadata.
    /// The host the runner executes on.
    pub runner_host: Option<<L as Lookup<RunnerHost>>::Index>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
