// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use perfect_derive::perfect_derive;

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
#[derive(Builder)]
#[perfect_derive(Debug, Clone)]
#[builder(pattern = "owned")]
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
    #[builder(default, setter(into))]
    pub source_branch: String,
    /// The `HEAD` commit of the merge request.
    #[builder(default, setter(into))]
    pub sha: String,
    /// The target project.
    pub target_project: <L as Lookup<Project<L>>>::Index,
    /// The target branch.
    #[builder(default, setter(into))]
    pub target_branch: String,

    // Forge metadata.
    /// The id of the merge request.
    pub forge_id: u64,
    /// The title of the merge request.
    #[builder(default, setter(into))]
    pub title: String,
    /// The description of the merge request.
    #[builder(default, setter(into))]
    pub description: String,
    /// The state of the merge request.
    pub state: MergeRequestStatus,
    /// The author of the merge request.
    pub author: <L as Lookup<User<L>>>::Index,
    /// The URL of the pipeline webpage.
    #[builder(setter(into))]
    pub url: String,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl<L> MergeRequest<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
    L: Lookup<User<L>>,
{
    /// Create a builder for the structure.
    pub fn builder() -> MergeRequestBuilder<L> {
        MergeRequestBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{
        Instance, MergeRequest, MergeRequestBuilderError, MergeRequestStatus, Project, User,
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

    fn user(instance: <TestLookup as Lookup<Instance>>::Index) -> User<TestLookup> {
        User::builder()
            .forge_id(0)
            .instance(instance)
            .build()
            .unwrap()
    }

    #[test]
    fn id_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = MergeRequest::<TestLookup>::builder()
            .source_project(proj_idx.clone())
            .target_project(proj_idx)
            .forge_id(0)
            .state(MergeRequestStatus::Open)
            .author(user_idx)
            .url("url")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, MergeRequestBuilderError, "id");
    }

    #[test]
    fn source_project_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = MergeRequest::<TestLookup>::builder()
            .id(0)
            .target_project(proj_idx)
            .forge_id(0)
            .state(MergeRequestStatus::Open)
            .author(user_idx)
            .url("url")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, MergeRequestBuilderError, "source_project");
    }

    #[test]
    fn target_project_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = MergeRequest::<TestLookup>::builder()
            .id(0)
            .source_project(proj_idx)
            .forge_id(0)
            .state(MergeRequestStatus::Open)
            .author(user_idx)
            .url("url")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, MergeRequestBuilderError, "target_project");
    }

    #[test]
    fn forge_id_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = MergeRequest::<TestLookup>::builder()
            .id(0)
            .source_project(proj_idx.clone())
            .target_project(proj_idx)
            .state(MergeRequestStatus::Open)
            .author(user_idx)
            .url("url")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, MergeRequestBuilderError, "forge_id");
    }

    #[test]
    fn state_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = MergeRequest::<TestLookup>::builder()
            .id(0)
            .source_project(proj_idx.clone())
            .target_project(proj_idx)
            .forge_id(0)
            .author(user_idx)
            .url("url")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, MergeRequestBuilderError, "state");
    }

    #[test]
    fn author_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let proj_idx = lookup.store(proj);

        let err = MergeRequest::<TestLookup>::builder()
            .id(0)
            .source_project(proj_idx.clone())
            .target_project(proj_idx)
            .forge_id(0)
            .state(MergeRequestStatus::Open)
            .url("url")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, MergeRequestBuilderError, "author");
    }

    #[test]
    fn url_is_required() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        let err = MergeRequest::<TestLookup>::builder()
            .id(0)
            .source_project(proj_idx.clone())
            .target_project(proj_idx)
            .forge_id(0)
            .state(MergeRequestStatus::Open)
            .author(user_idx)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, MergeRequestBuilderError, "url");
    }

    #[test]
    fn sufficient_fields() {
        let mut lookup = TestLookup::default();
        let proj = project(&mut lookup);
        let user = user(proj.instance.clone());
        let proj_idx = lookup.store(proj);
        let user_idx = lookup.store(user);

        MergeRequest::<TestLookup>::builder()
            .id(0)
            .source_project(proj_idx.clone())
            .target_project(proj_idx)
            .forge_id(0)
            .state(MergeRequestStatus::Open)
            .author(user_idx)
            .url("url")
            .build()
            .unwrap();
    }
}
