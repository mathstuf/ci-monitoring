// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use perfect_derive::perfect_derive;

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
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[non_exhaustive]
pub struct Runner<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
    L: Lookup<RunnerHost>,
{
    // Metadata.
    /// The description of the runner.
    #[builder(default, setter(into))]
    pub description: String,
    /// The runner type.
    pub runner_type: RunnerType,
    /// The maximum timeout for jobs on the runner (in seconds).
    #[builder(default, setter(into))]
    pub maximum_timeout: Option<u64>,
    /// Protection level of refs that can use this runner.
    pub protection_level: RunnerProtectionLevel,

    // Runner program metadata.
    /// The implementation of the runner.
    #[builder(default, setter(into))]
    pub implementation: String,
    /// The version of the runner.
    #[builder(default, setter(into))]
    pub version: String,
    /// The revision of the runner.
    #[builder(default, setter(into))]
    pub revision: String,
    /// The platform of the runner.
    #[builder(default, setter(into))]
    pub platform: String,
    /// The CPU architecture of the runner.
    #[builder(default, setter(into))]
    pub architecture: String,

    // Scheduling metadata.
    /// The tags for the runner.
    #[builder(default, setter(into))]
    pub tags: Vec<String>,
    /// Whether untagged jobs may use this runner.
    #[builder(default)]
    pub run_untagged: bool,
    /// The set of projects which may use this runner.
    #[builder(default, setter(into))]
    pub projects: Vec<<L as Lookup<Project<L>>>::Index>,

    // Forge metadata.
    /// The id of the runner.
    pub forge_id: u64,
    /// Whether the runner is paused or not.
    #[builder(default)]
    pub paused: bool,
    /// Whether the runner is shared with other projects or not.
    #[builder(default)]
    pub shared: bool,
    /// Whether the runner is online or not.
    #[builder(default)]
    pub online: bool,
    /// Whether the runner is locked to its projects or not.
    #[builder(default)]
    pub locked: bool,
    /// When the runner last contacted the forge.
    #[builder(default, setter(into))]
    pub contacted_at: Option<DateTime<Utc>>,
    /// The maintenance note of the runner.
    #[builder(default, setter(into))]
    pub maintenance_note: Option<String>,
    /// The instance for which the runner performs jobs.
    pub instance: <L as Lookup<Instance>>::Index,

    // Maintenance metadata.
    /// The host the runner executes on.
    #[builder(default, setter(into))]
    pub runner_host: Option<<L as Lookup<RunnerHost>>::Index>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl<L> Runner<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
    L: Lookup<RunnerHost>,
    L: Clone,
{
    /// Create a builder for the structure.
    pub fn builder() -> RunnerBuilder<L> {
        RunnerBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{Instance, Runner, RunnerBuilderError, RunnerProtectionLevel, RunnerType};
    use crate::Lookup;

    use crate::test::TestLookup;

    fn instance() -> Instance {
        Instance::builder()
            .unique_id(0)
            .forge("forge")
            .url("url")
            .build()
            .unwrap()
    }

    #[test]
    fn forge_id_is_required() {
        let mut lookup = TestLookup::default();
        let inst = instance();
        let idx = lookup.store(inst);

        let err = Runner::<TestLookup>::builder()
            .instance(idx)
            .runner_type(RunnerType::Instance)
            .protection_level(RunnerProtectionLevel::Any)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, RunnerBuilderError, "forge_id");
    }

    #[test]
    fn instance_is_required() {
        let err = Runner::<TestLookup>::builder()
            .forge_id(0)
            .runner_type(RunnerType::Instance)
            .protection_level(RunnerProtectionLevel::Any)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, RunnerBuilderError, "instance");
    }

    #[test]
    fn runner_type_is_required() {
        let mut lookup = TestLookup::default();
        let inst = instance();
        let idx = lookup.store(inst);

        let err = Runner::<TestLookup>::builder()
            .forge_id(0)
            .instance(idx)
            .protection_level(RunnerProtectionLevel::Any)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, RunnerBuilderError, "runner_type");
    }

    #[test]
    fn protection_level_is_required() {
        let mut lookup = TestLookup::default();
        let inst = instance();
        let idx = lookup.store(inst);

        let err = Runner::<TestLookup>::builder()
            .forge_id(0)
            .instance(idx)
            .runner_type(RunnerType::Instance)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, RunnerBuilderError, "protection_level");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let inst = instance();
        let idx = lookup.store(inst);

        Runner::<TestLookup>::builder()
            .forge_id(0)
            .instance(idx)
            .runner_type(RunnerType::Instance)
            .protection_level(RunnerProtectionLevel::Any)
            .build()
            .unwrap();
    }
}
