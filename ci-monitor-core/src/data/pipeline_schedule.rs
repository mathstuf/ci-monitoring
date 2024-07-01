// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use perfect_derive::perfect_derive;

use crate::data::{Instance, PipelineVariables, Project, User};
use crate::Lookup;

/// A pipeline schedule.
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[builder(pattern = "owned")]
#[non_exhaustive]
pub struct PipelineSchedule<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    // Metadata.
    /// The name of the pipeline schedule.
    #[builder(default, setter(into))]
    pub name: String,

    // Repository metadata.
    /// The project the schedule is associated with.
    pub project: <L as Lookup<Project<L>>>::Index,
    /// The ref the pipeline builds when it builds.
    #[builder(setter(into))]
    pub ref_: String,

    // Execution metadata.
    /// Variables the schedule makes available to pipelines it starts.
    #[builder(default)]
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
    #[builder(default)]
    pub active: bool,
    /// When the schedule will next create a pipeline.
    #[builder(default)]
    pub next_run: Option<DateTime<Utc>>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl<L> PipelineSchedule<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    /// Create a builder for the structure.
    pub fn builder() -> PipelineScheduleBuilder<L> {
        PipelineScheduleBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::data::{Instance, PipelineSchedule, PipelineScheduleBuilderError, Project, User};
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

    fn user(instance: <TestLookup as Lookup<Instance>>::Index) -> User<TestLookup> {
        User::builder()
            .forge_id(0)
            .instance(instance)
            .build()
            .unwrap()
    }

    #[test]
    fn project_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let user_idx = lookup.store(user);

        let err = PipelineSchedule::<TestLookup>::builder()
            .ref_("main")
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .owner(user_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineScheduleBuilderError, "project");
    }

    #[test]
    fn ref_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = PipelineSchedule::<TestLookup>::builder()
            .project(proj_idx)
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .owner(user_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineScheduleBuilderError, "ref_");
    }

    #[test]
    fn forge_id_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = PipelineSchedule::<TestLookup>::builder()
            .project(proj_idx)
            .ref_("main")
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .owner(user_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineScheduleBuilderError, "forge_id");
    }

    #[test]
    fn created_at_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = PipelineSchedule::<TestLookup>::builder()
            .project(proj_idx)
            .ref_("main")
            .forge_id(0)
            .updated_at(Utc::now())
            .owner(user_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineScheduleBuilderError, "created_at");
    }

    #[test]
    fn updated_at_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = PipelineSchedule::<TestLookup>::builder()
            .project(proj_idx)
            .ref_("main")
            .forge_id(0)
            .created_at(Utc::now())
            .owner(user_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineScheduleBuilderError, "updated_at");
    }

    #[test]
    fn owner_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = PipelineSchedule::<TestLookup>::builder()
            .project(proj_idx)
            .ref_("main")
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineScheduleBuilderError, "owner");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        PipelineSchedule::<TestLookup>::builder()
            .project(proj_idx)
            .ref_("main")
            .forge_id(0)
            .created_at(Utc::now())
            .updated_at(Utc::now())
            .owner(user_idx)
            .build()
            .unwrap();
    }
}
