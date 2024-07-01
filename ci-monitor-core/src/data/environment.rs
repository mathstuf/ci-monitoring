// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use perfect_derive::perfect_derive;

use crate::data::{Instance, Project};
use crate::Lookup;

/// The state of an environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvironmentState {
    /// The environment is available.
    Available,
    /// The environment is shutting down.
    Stopping,
    /// The environment is stopped.
    Stopped,
}

/// The environment tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvironmentTier {
    /// An environment intended for production.
    Production,
    /// An environment for staging before production.
    Staging,
    /// An environment for testing.
    Testing,
    /// An environment for development.
    Development,
    /// An environment for other purposes.
    Other,
}

/// An environment into which deployments may be made.
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[builder(pattern = "owned")]
#[non_exhaustive]
pub struct Environment<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
{
    // Metadata.
    /// The name of the environment.
    #[builder(setter(into))]
    pub name: String,
    /// The external URL of the environment.
    #[builder(default, setter(into))]
    pub external_url: String,
    /// The state of the environment.
    pub state: EnvironmentState,
    /// The tier of the environment.
    pub tier: EnvironmentTier,

    // Forge metadata.
    /// The ID of the environment.
    pub forge_id: u64,
    /// The project the environment is for.
    pub project: <L as Lookup<Project<L>>>::Index,
    /// When the environment was created.
    pub created_at: DateTime<Utc>,
    /// When the environment was updated.
    pub updated_at: DateTime<Utc>,
    /// When the environment will automatically stop.
    #[builder(default)]
    pub auto_stop_at: Option<DateTime<Utc>>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl<L> Environment<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
{
    /// Create a builder for the structure.
    pub fn builder() -> EnvironmentBuilder<L> {
        EnvironmentBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::data::{
        Environment, EnvironmentBuilderError, EnvironmentState, EnvironmentTier, Instance, Project,
    };
    use crate::Lookup;

    use crate::test::TestLookup;

    fn project(lookup: &mut TestLookup) -> Project<TestLookup> {
        let instance = Instance::builder()
            .unique_id(0)
            .forge("forge")
            .url("url")
            .build()
            .unwrap();
        let idx = lookup.store(instance);

        Project::builder()
            .forge_id(0)
            .instance(idx)
            .build()
            .unwrap()
    }

    #[test]
    fn name_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Environment::<TestLookup>::builder()
            .state(EnvironmentState::Available)
            .tier(EnvironmentTier::Testing)
            .forge_id(0)
            .project(proj_idx)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, EnvironmentBuilderError, "name");
    }

    #[test]
    fn state_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Environment::<TestLookup>::builder()
            .name("name")
            .tier(EnvironmentTier::Testing)
            .forge_id(0)
            .project(proj_idx)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, EnvironmentBuilderError, "state");
    }

    #[test]
    fn tier_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Environment::<TestLookup>::builder()
            .name("name")
            .state(EnvironmentState::Available)
            .forge_id(0)
            .project(proj_idx)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, EnvironmentBuilderError, "tier");
    }

    #[test]
    fn forge_id_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Environment::<TestLookup>::builder()
            .name("name")
            .state(EnvironmentState::Available)
            .tier(EnvironmentTier::Testing)
            .project(proj_idx)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, EnvironmentBuilderError, "forge_id");
    }

    #[test]
    fn project_is_required() {
        let err = Environment::<TestLookup>::builder()
            .name("name")
            .state(EnvironmentState::Available)
            .tier(EnvironmentTier::Testing)
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, EnvironmentBuilderError, "project");
    }

    #[test]
    fn created_at_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Environment::<TestLookup>::builder()
            .name("name")
            .state(EnvironmentState::Available)
            .tier(EnvironmentTier::Testing)
            .forge_id(0)
            .project(proj_idx)
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, EnvironmentBuilderError, "created_at");
    }

    #[test]
    fn updated_at_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = Environment::<TestLookup>::builder()
            .name("name")
            .state(EnvironmentState::Available)
            .tier(EnvironmentTier::Testing)
            .forge_id(0)
            .project(proj_idx)
            .created_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, EnvironmentBuilderError, "updated_at");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        Environment::<TestLookup>::builder()
            .name("name")
            .state(EnvironmentState::Available)
            .tier(EnvironmentTier::Testing)
            .forge_id(0)
            .project(proj_idx)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap();
    }
}
