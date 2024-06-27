// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use perfect_derive::perfect_derive;

use crate::data::{BlobReference, Instance};
use crate::Lookup;

/// A user account on an instance.
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[builder(pattern = "owned")]
#[non_exhaustive]
pub struct User<L>
where
    L: Lookup<Instance>,
{
    /// The handle of the user.
    #[builder(default, setter(into))]
    pub handle: String,
    /// The display name of the user.
    #[builder(default, setter(into))]
    pub name: String,
    /// The email address of the user.
    #[builder(default, setter(into))]
    pub email: Option<String>,
    /// The avatar of the user.
    #[builder(default, setter(into))]
    pub avatar: Option<BlobReference>,

    // Forge metadata.
    /// The ID of the user.
    pub forge_id: u64,
    /// The instance the user account is associated with.
    pub instance: <L as Lookup<Instance>>::Index,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl<L> User<L>
where
    L: Lookup<Instance> + Clone,
{
    /// Create a builder for the structure.
    pub fn builder() -> UserBuilder<L> {
        UserBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{Instance, User, UserBuilderError};
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

        let err = User::<TestLookup>::builder()
            .instance(idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, UserBuilderError, "forge_id");
    }

    #[test]
    fn instance_is_required() {
        let err = User::<TestLookup>::builder()
            .forge_id(0)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, UserBuilderError, "instance");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let inst = instance();
        let idx = lookup.store(inst);

        User::<TestLookup>::builder()
            .forge_id(0)
            .instance(idx)
            .build()
            .unwrap();
    }
}
