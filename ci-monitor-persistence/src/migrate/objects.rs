// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use ci_monitor_core::Lookup;
use perfect_derive::perfect_derive;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("dangling source index type {}: '{}'", type_, index)]
    DanglingSourceIndex { type_: &'static str, index: String },
    #[error("duplicate source index of type {}: '{}'", type_, index)]
    DuplicateSourceIndex { type_: &'static str, index: String },
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

/// Migrate an object store's objects into another store.
pub fn migrate_object_store<Source, Sink>(
    source: &Source,
    sink: &mut Sink,
) -> Result<(), MigrationError> {
    // Deployments
    // Environments
    // Instances
    // Job artifacts
    // Jobs
    // Merge requests
    // Pipeline schedules
    // Pipelines
    // Projects
    // Runner hosts
    // Runners
    // Users

    Ok(())
}
