// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use perfect_derive::perfect_derive;

use crate::data::Instance;
use crate::Lookup;

/// An instance of a project.
///
/// This represents an instance of a project. There may be multiple instances of the project on
/// different instances or even on a given instance.
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[builder(pattern = "owned")]
#[non_exhaustive]
pub struct Project<L>
where
    L: Lookup<Instance>,
{
    // Metadata.
    /// The name of the project.
    ///
    /// This is informal and a project may exist at multiple locations.
    #[builder(default, setter(into))]
    pub name: String,

    // Forge metadata.
    /// The ID of the project.
    pub forge_id: u64,
    /// The URL of the project.
    #[builder(default, setter(into))]
    pub url: String,
    /// The instance on which the project lives.
    pub instance: <L as Lookup<Instance>>::Index,
    /// The path to the repository on the instance.
    #[builder(default, setter(into))]
    pub instance_path: String,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl<L> Project<L>
where
    L: Lookup<Instance> + Clone,
{
    /// Create a builder for the structure.
    pub fn builder() -> ProjectBuilder<L> {
        ProjectBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{Instance, Project, ProjectBuilderError};
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

        let err = Project::<TestLookup>::builder()
            .instance(idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, ProjectBuilderError, "forge_id");
    }

    #[test]
    fn instance_is_required() {
        let err = Project::<TestLookup>::builder()
            .forge_id(0)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, ProjectBuilderError, "instance");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let inst = instance();
        let idx = lookup.store(inst);

        Project::<TestLookup>::builder()
            .forge_id(0)
            .instance(idx)
            .build()
            .unwrap();
    }
}
