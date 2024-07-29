// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use ci_monitor_core::data::{Instance, Project, RunnerHost, User};
use ci_monitor_core::Lookup;
use perfect_derive::perfect_derive;
use thiserror::Error;

use crate::DiscoverableLookup;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("dangling source index type {}: '{}'", type_, index)]
    DanglingSourceIndex { type_: &'static str, index: String },
    #[error("duplicate source index of type {}: '{}'", type_, index)]
    DuplicateSourceIndex { type_: &'static str, index: String },
    #[error("missing source data of type {} at index '{}'", type_, index)]
    MissingData { type_: &'static str, index: String },
}

impl MigrationError {
    fn dangling_source_index<L, T>(index: &<L as Lookup<T>>::Index) -> Self
    where
        L: Lookup<T>,
    {
        Self::DanglingSourceIndex {
            type_: any::type_name::<T>(),
            index: format!("{:?}", index),
        }
    }

    fn duplicate_source_index<L, T>(index: &<L as Lookup<T>>::Index) -> Self
    where
        L: Lookup<T>,
    {
        Self::DuplicateSourceIndex {
            type_: any::type_name::<T>(),
            index: format!("{:?}", index),
        }
    }

    fn missing_data<L, T>(index: &<L as Lookup<T>>::Index) -> Self
    where
        L: Lookup<T>,
    {
        Self::MissingData {
            type_: any::type_name::<T>(),
            index: format!("{:?}", index),
        }
    }
}

#[perfect_derive(Default)]
struct IndexMap<Source, Sink, T, U = T>
where
    Source: Lookup<T>,
    Sink: Lookup<U>,
{
    map: BTreeMap<<Source as Lookup<T>>::Index, <Sink as Lookup<U>>::Index>,
}

type IndexEntry<'a, Source, Sink, T, U = T> =
    Entry<'a, <Source as Lookup<T>>::Index, <Sink as Lookup<U>>::Index>;

impl<Source, Sink, T, U> IndexMap<Source, Sink, T, U>
where
    Source: Lookup<T>,
    <Source as Lookup<T>>::Index: Ord,
    Sink: Lookup<U>,
{
    fn contains_key(&self, key: &<Source as Lookup<T>>::Index) -> bool {
        self.map.contains_key(key)
    }

    fn get(
        &self,
        key: &<Source as Lookup<T>>::Index,
    ) -> Result<<Sink as Lookup<U>>::Index, MigrationError> {
        if let Some(sink_idx) = self.map.get(key) {
            Ok(sink_idx.clone())
        } else {
            Err(MigrationError::dangling_source_index::<Source, T>(key))
        }
    }

    fn entry(
        &mut self,
        key: <Source as Lookup<T>>::Index,
    ) -> Result<IndexEntry<Source, Sink, T, U>, MigrationError> {
        let entry = self.map.entry(key);
        if matches!(entry, Entry::Occupied(_)) {
            Ok(entry)
        } else {
            Err(MigrationError::duplicate_source_index::<Source, T>(
                entry.key(),
            ))
        }
    }
}

trait Migration<Source, Sink, T, U>
where
    Source: DiscoverableLookup<T>,
    <Source as Lookup<T>>::Index: Ord,
    Sink: DiscoverableLookup<U>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, T, U>,
    ) -> Result<(), MigrationError>;
}

fn get_data<Source, T>(
    source: &Source,
    idx: &<Source as Lookup<T>>::Index,
) -> Result<T, MigrationError>
where
    Source: Lookup<T>,
    T: Clone,
{
    source
        .lookup(idx)
        .ok_or_else(|| MigrationError::missing_data::<Source, T>(idx))
        .cloned()
}

struct InstanceMigration {}

impl<Source, Sink> Migration<Source, Sink, Instance, Instance> for InstanceMigration
where
    Source: DiscoverableLookup<Instance>,
    <Source as Lookup<Instance>>::Index: Ord,
    Sink: DiscoverableLookup<Instance>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, Instance, Instance>,
    ) -> Result<(), MigrationError> {
        for idx in source.all_indices() {
            let entry = imap.entry(idx)?;
            let data = get_data(source, entry.key())?;

            // TODO: check if the sink already has this `Instance`.

            let new_index = sink.store(data.clone());
            entry.or_insert(new_index);
        }

        Ok(())
    }
}

struct RunnerHostMigration {}

impl<Source, Sink> Migration<Source, Sink, RunnerHost, RunnerHost> for RunnerHostMigration
where
    Source: DiscoverableLookup<RunnerHost>,
    <Source as Lookup<RunnerHost>>::Index: Ord,
    Sink: DiscoverableLookup<RunnerHost>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, RunnerHost, RunnerHost>,
    ) -> Result<(), MigrationError> {
        for idx in source.all_indices() {
            let entry = imap.entry(idx)?;
            let data = get_data(source, entry.key())?;

            // TODO: check if the sink already has this `RunnerHost`.

            let new_index = sink.store(data.clone());
            entry.or_insert(new_index);
        }

        Ok(())
    }
}

struct UserMigration<'a, Source, Sink>
where
    Source: Lookup<Instance>,
    Sink: Lookup<Instance>,
{
    instances: &'a IndexMap<Source, Sink, Instance>,
}

impl<'a, Source, Sink> Migration<Source, Sink, User<Source>, User<Sink>>
    for UserMigration<'a, Source, Sink>
where
    Source: DiscoverableLookup<User<Source>>,
    Source: Lookup<Instance>,
    <Source as Lookup<Instance>>::Index: Ord,
    <Source as Lookup<User<Source>>>::Index: Ord,
    Sink: DiscoverableLookup<User<Sink>>,
    Sink: Lookup<Instance>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, User<Source>, User<Sink>>,
    ) -> Result<(), MigrationError> {
        for idx in source.all_indices() {
            let entry = imap.entry(idx)?;
            let data: User<Source> = get_data(source, entry.key())?;

            // TODO: check if the sink already has this `User`.

            let mut new_data: User<Sink> = User::builder()
                .forge_id(data.forge_id)
                .instance(self.instances.get(&data.instance)?)
                .build()
                .unwrap();
            new_data.handle = data.handle;
            new_data.name = data.name;
            new_data.email = data.email;
            new_data.avatar = data.avatar;
            new_data.cim_fetched_at = data.cim_fetched_at;
            new_data.cim_refreshed_at = data.cim_refreshed_at;

            let new_index = sink.store(new_data);
            entry.or_insert(new_index);
        }

        Ok(())
    }
}

struct ProjectMigration<'a, Source, Sink>
where
    Source: Lookup<Instance>,
    Sink: Lookup<Instance>,
{
    instances: &'a IndexMap<Source, Sink, Instance>,
}

impl<'a, Source, Sink> Migration<Source, Sink, Project<Source>, Project<Sink>>
    for ProjectMigration<'a, Source, Sink>
where
    Source: DiscoverableLookup<Project<Source>>,
    Source: Lookup<Instance>,
    <Source as Lookup<Instance>>::Index: Ord,
    <Source as Lookup<Project<Source>>>::Index: Ord,
    Sink: DiscoverableLookup<Project<Sink>>,
    Sink: Lookup<Instance>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, Project<Source>, Project<Sink>>,
    ) -> Result<(), MigrationError> {
        for idx in source.all_indices() {
            let entry = imap.entry(idx)?;
            let data: Project<Source> = get_data(source, entry.key())?;

            // TODO: check if the sink already has this `Project`.

            let mut new_data: Project<Sink> = Project::builder()
                .forge_id(data.forge_id)
                .instance(self.instances.get(&data.instance)?)
                .build()
                .unwrap();
            new_data.name = data.name;
            new_data.url = data.url;
            new_data.instance_path = data.instance_path;
            new_data.cim_fetched_at = data.cim_fetched_at;
            new_data.cim_refreshed_at = data.cim_refreshed_at;

            let new_index = sink.store(new_data);
            entry.or_insert(new_index);
        }

        Ok(())
    }
}

/// Migrate an object store's objects into another store.
pub fn migrate_object_store<Source, Sink>(
    source: &Source,
    sink: &mut Sink,
) -> Result<(), MigrationError>
where
    Source: DiscoverableLookup<Instance>,
    Source: DiscoverableLookup<Project<Source>>,
    Source: DiscoverableLookup<RunnerHost>,
    Source: DiscoverableLookup<User<Source>>,
    <Source as Lookup<Instance>>::Index: Ord,
    <Source as Lookup<Project<Source>>>::Index: Ord,
    <Source as Lookup<RunnerHost>>::Index: Ord,
    <Source as Lookup<User<Source>>>::Index: Ord,
    Sink: DiscoverableLookup<Instance>,
    Sink: DiscoverableLookup<Project<Sink>>,
    Sink: DiscoverableLookup<RunnerHost>,
    Sink: DiscoverableLookup<User<Sink>>,
{
    // Instances
    let mut instance_map = IndexMap::<Source, Sink, Instance>::default();
    {
        let migration = InstanceMigration {};
        migration.migrate(source, sink, &mut instance_map)?;
    }

    // Runner hosts
    let mut runner_host_map = IndexMap::<Source, Sink, RunnerHost>::default();
    {
        let migration = RunnerHostMigration {};
        migration.migrate(source, sink, &mut runner_host_map)?;
    }

    // Users
    let mut user_map = IndexMap::<Source, Sink, User<Source>, User<Sink>>::default();
    {
        let migration = UserMigration {
            instances: &mut instance_map,
        };
        migration.migrate(source, sink, &mut user_map)?;
    }

    // Projects
    let mut project_map = IndexMap::<Source, Sink, Project<Source>, Project<Sink>>::default();
    {
        let migration = ProjectMigration {
            instances: &mut instance_map,
        };
        migration.migrate(source, sink, &mut project_map)?;
    }

    // Deployments
    // Environments
    // Job artifacts
    // Jobs
    // Merge requests
    // Pipeline schedules
    // Pipelines
    // Runners

    Ok(())
}
