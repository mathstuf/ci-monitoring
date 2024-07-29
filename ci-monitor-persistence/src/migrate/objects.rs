// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::any;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::mem;

use ci_monitor_core::data::{
    Deployment, Environment, Instance, MergeRequest, Pipeline, PipelineSchedule, Project, Runner,
    RunnerHost, User,
};
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

struct RunnerMigration<'a, Source, Sink>
where
    Source: Lookup<Instance>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<RunnerHost>,
    Sink: Lookup<Instance>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<RunnerHost>,
{
    instances: &'a IndexMap<Source, Sink, Instance>,
    projects: &'a IndexMap<Source, Sink, Project<Source>, Project<Sink>>,
    runner_hosts: &'a IndexMap<Source, Sink, RunnerHost>,
}

impl<'a, Source, Sink> Migration<Source, Sink, Runner<Source>, Runner<Sink>>
    for RunnerMigration<'a, Source, Sink>
where
    Source: DiscoverableLookup<Runner<Source>>,
    Source: Lookup<Instance>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<RunnerHost>,
    <Source as Lookup<Instance>>::Index: Ord,
    <Source as Lookup<Project<Source>>>::Index: Ord,
    <Source as Lookup<Runner<Source>>>::Index: Ord,
    <Source as Lookup<RunnerHost>>::Index: Ord,
    Sink: DiscoverableLookup<Runner<Sink>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<RunnerHost>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, Runner<Source>, Runner<Sink>>,
    ) -> Result<(), MigrationError> {
        for idx in source.all_indices() {
            let entry = imap.entry(idx)?;
            let data: Runner<Source> = get_data(source, entry.key())?;

            // TODO: check if the sink already has this `Runner`.

            let mut new_data: Runner<Sink> = Runner::builder()
                .forge_id(data.forge_id)
                .instance(self.instances.get(&data.instance)?)
                .runner_type(data.runner_type)
                .protection_level(data.protection_level)
                .build()
                .unwrap();
            new_data.description = data.description;
            new_data.maximum_timeout = data.maximum_timeout;
            new_data.implementation = data.implementation;
            new_data.version = data.version;
            new_data.revision = data.revision;
            new_data.platform = data.platform;
            new_data.architecture = data.architecture;
            new_data.tags = data.tags;
            new_data.run_untagged = data.run_untagged;
            new_data.projects = data
                .projects
                .iter()
                .map(|idx| self.projects.get(idx))
                .collect::<Result<Vec<_>, _>>()?;
            new_data.paused = data.paused;
            new_data.shared = data.shared;
            new_data.online = data.online;
            new_data.locked = data.locked;
            new_data.contacted_at = data.contacted_at;
            new_data.maintenance_note = data.maintenance_note;
            new_data.runner_host = data
                .runner_host
                .map(|idx| self.runner_hosts.get(&idx))
                .transpose()?;
            new_data.cim_fetched_at = data.cim_fetched_at;
            new_data.cim_refreshed_at = data.cim_refreshed_at;

            let new_index = sink.store(new_data);
            entry.or_insert(new_index);
        }

        Ok(())
    }
}

struct MergeRequestMigration<'a, Source, Sink>
where
    Source: Lookup<Instance>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<User<Source>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<User<Sink>>,
{
    projects: &'a IndexMap<Source, Sink, Project<Source>, Project<Sink>>,
    users: &'a IndexMap<Source, Sink, User<Source>, User<Sink>>,
}

impl<'a, Source, Sink> Migration<Source, Sink, MergeRequest<Source>, MergeRequest<Sink>>
    for MergeRequestMigration<'a, Source, Sink>
where
    Source: DiscoverableLookup<MergeRequest<Source>>,
    Source: Lookup<Instance>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<User<Source>>,
    <Source as Lookup<Project<Source>>>::Index: Ord,
    <Source as Lookup<MergeRequest<Source>>>::Index: Ord,
    <Source as Lookup<User<Source>>>::Index: Ord,
    Sink: DiscoverableLookup<MergeRequest<Sink>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<User<Sink>>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, MergeRequest<Source>, MergeRequest<Sink>>,
    ) -> Result<(), MigrationError> {
        for idx in source.all_indices() {
            let entry = imap.entry(idx)?;
            let data: MergeRequest<Source> = get_data(source, entry.key())?;

            // TODO: check if the sink already has this `MergeRequest`.

            let mut new_data: MergeRequest<Sink> = MergeRequest::builder()
                .id(data.id)
                .source_project(self.projects.get(&data.source_project)?)
                .target_project(self.projects.get(&data.target_project)?)
                .forge_id(data.forge_id)
                .state(data.state)
                .author(self.users.get(&data.author)?)
                .url(data.url)
                .build()
                .unwrap();
            new_data.source_branch = data.source_branch;
            new_data.sha = data.sha;
            new_data.target_branch = data.target_branch;
            new_data.title = data.title;
            new_data.description = data.description;
            new_data.cim_fetched_at = data.cim_fetched_at;
            new_data.cim_refreshed_at = data.cim_refreshed_at;

            let new_index = sink.store(new_data);
            entry.or_insert(new_index);
        }

        Ok(())
    }
}

struct PipelineScheduleMigration<'a, Source, Sink>
where
    Source: Lookup<Instance>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<User<Source>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<User<Sink>>,
{
    projects: &'a IndexMap<Source, Sink, Project<Source>, Project<Sink>>,
    users: &'a IndexMap<Source, Sink, User<Source>, User<Sink>>,
}

impl<'a, Source, Sink> Migration<Source, Sink, PipelineSchedule<Source>, PipelineSchedule<Sink>>
    for PipelineScheduleMigration<'a, Source, Sink>
where
    Source: DiscoverableLookup<PipelineSchedule<Source>>,
    Source: Lookup<Instance>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<User<Source>>,
    <Source as Lookup<PipelineSchedule<Source>>>::Index: Ord,
    <Source as Lookup<Project<Source>>>::Index: Ord,
    <Source as Lookup<User<Source>>>::Index: Ord,
    Sink: DiscoverableLookup<PipelineSchedule<Sink>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<User<Sink>>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, PipelineSchedule<Source>, PipelineSchedule<Sink>>,
    ) -> Result<(), MigrationError> {
        for idx in source.all_indices() {
            let entry = imap.entry(idx)?;
            let data: PipelineSchedule<Source> = get_data(source, entry.key())?;

            // TODO: check if the sink already has this `PipelineSchedule`.

            let mut new_data: PipelineSchedule<Sink> = PipelineSchedule::builder()
                .project(self.projects.get(&data.project)?)
                .ref_(data.ref_)
                .forge_id(data.forge_id)
                .created_at(data.created_at)
                .updated_at(data.updated_at)
                .owner(self.users.get(&data.owner)?)
                .build()
                .unwrap();
            new_data.name = data.name;
            new_data.variables = data.variables;
            new_data.active = data.active;
            new_data.next_run = data.next_run;
            new_data.cim_fetched_at = data.cim_fetched_at;
            new_data.cim_refreshed_at = data.cim_refreshed_at;

            let new_index = sink.store(new_data);
            entry.or_insert(new_index);
        }

        Ok(())
    }
}

struct PipelineMigration<'a, Source, Sink>
where
    Source: Lookup<Instance>,
    Source: Lookup<MergeRequest<Source>>,
    Source: Lookup<PipelineSchedule<Source>>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<User<Source>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<MergeRequest<Sink>>,
    Sink: Lookup<PipelineSchedule<Sink>>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<User<Sink>>,
{
    merge_requests: &'a IndexMap<Source, Sink, MergeRequest<Source>, MergeRequest<Sink>>,
    pipeline_schedules:
        &'a IndexMap<Source, Sink, PipelineSchedule<Source>, PipelineSchedule<Sink>>,
    projects: &'a IndexMap<Source, Sink, Project<Source>, Project<Sink>>,
    users: &'a IndexMap<Source, Sink, User<Source>, User<Sink>>,
}

impl<'a, Source, Sink> Migration<Source, Sink, Pipeline<Source>, Pipeline<Sink>>
    for PipelineMigration<'a, Source, Sink>
where
    Source: DiscoverableLookup<Pipeline<Source>>,
    Source: Lookup<Instance>,
    Source: Lookup<MergeRequest<Source>>,
    Source: Lookup<PipelineSchedule<Source>>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<User<Source>>,
    <Source as Lookup<MergeRequest<Source>>>::Index: Ord,
    <Source as Lookup<Pipeline<Source>>>::Index: Ord,
    <Source as Lookup<PipelineSchedule<Source>>>::Index: Ord,
    <Source as Lookup<Project<Source>>>::Index: Ord,
    <Source as Lookup<User<Source>>>::Index: Ord,
    Sink: DiscoverableLookup<Pipeline<Sink>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<MergeRequest<Sink>>,
    Sink: Lookup<PipelineSchedule<Sink>>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<User<Sink>>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, Pipeline<Source>, Pipeline<Sink>>,
    ) -> Result<(), MigrationError> {
        let mut with_missing_parent = BTreeSet::new();
        let mut pipelines_to_inspect = source.all_indices();

        while !pipelines_to_inspect.is_empty() {
            for idx in pipelines_to_inspect.drain(..) {
                let data: Pipeline<Source> = {
                    let entry = imap.entry(idx.clone())?;
                    get_data(source, entry.key())?
                };

                if let Some(parent_pipeline) = data.parent_pipeline.as_ref() {
                    if !imap.contains_key(parent_pipeline) {
                        with_missing_parent.insert(parent_pipeline.clone());
                        continue;
                    }
                }

                // TODO: check if the sink already has this `Pipeline`.

                let mut new_data: Pipeline<Sink> = Pipeline::builder()
                    .project(self.projects.get(&data.project)?)
                    .sha(data.sha)
                    .source(data.source)
                    .status(data.status)
                    .forge_id(data.forge_id)
                    .url(data.url)
                    .created_at(data.created_at)
                    .updated_at(data.updated_at)
                    .build()
                    .unwrap();
                new_data.name = data.name;
                new_data.previous_sha = data.previous_sha;
                new_data.refname = data.refname;
                new_data.stable_refname = data.stable_refname;
                new_data.schedule = data
                    .schedule
                    .map(|idx| self.pipeline_schedules.get(&idx))
                    .transpose()?;
                new_data.parent_pipeline =
                    data.parent_pipeline.map(|idx| imap.get(&idx)).transpose()?;
                new_data.merge_request = data
                    .merge_request
                    .map(|idx| self.merge_requests.get(&idx))
                    .transpose()?;
                new_data.variables = data.variables;
                new_data.user = data.user.map(|idx| self.users.get(&idx)).transpose()?;
                new_data.coverage = data.coverage;
                new_data.archived = data.archived;
                new_data.started_at = data.started_at;
                new_data.finished_at = data.finished_at;
                new_data.cim_fetched_at = data.cim_fetched_at;
                new_data.cim_refreshed_at = data.cim_refreshed_at;

                let new_index = sink.store(new_data);
                let entry = imap.entry(idx)?;
                entry.or_insert(new_index);
            }

            let swap = mem::take(&mut with_missing_parent);
            pipelines_to_inspect.extend(swap.into_iter());
        }

        Ok(())
    }
}

struct EnvironmentMigration<'a, Source, Sink>
where
    Source: Lookup<Instance>,
    Source: Lookup<Project<Source>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<Project<Sink>>,
{
    projects: &'a IndexMap<Source, Sink, Project<Source>, Project<Sink>>,
}

impl<'a, Source, Sink> Migration<Source, Sink, Environment<Source>, Environment<Sink>>
    for EnvironmentMigration<'a, Source, Sink>
where
    Source: DiscoverableLookup<Environment<Source>>,
    Source: Lookup<Instance>,
    Source: Lookup<Project<Source>>,
    <Source as Lookup<Environment<Source>>>::Index: Ord,
    <Source as Lookup<Project<Source>>>::Index: Ord,
    Sink: DiscoverableLookup<Environment<Sink>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<Project<Sink>>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, Environment<Source>, Environment<Sink>>,
    ) -> Result<(), MigrationError> {
        for idx in source.all_indices() {
            let entry = imap.entry(idx)?;
            let data: Environment<Source> = get_data(source, entry.key())?;

            // TODO: check if the sink already has this `Environment`.

            let mut new_data: Environment<Sink> = Environment::builder()
                .name(data.name)
                .state(data.state)
                .tier(data.tier)
                .forge_id(data.forge_id)
                .project(self.projects.get(&data.project)?)
                .created_at(data.created_at)
                .updated_at(data.updated_at)
                .build()
                .unwrap();
            new_data.external_url = data.external_url;
            new_data.auto_stop_at = data.auto_stop_at;
            new_data.cim_fetched_at = data.cim_fetched_at;
            new_data.cim_refreshed_at = data.cim_refreshed_at;

            let new_index = sink.store(new_data);
            entry.or_insert(new_index);
        }

        Ok(())
    }
}

struct DeploymentMigration<'a, Source, Sink>
where
    Source: Lookup<Environment<Source>>,
    Source: Lookup<Instance>,
    Source: Lookup<MergeRequest<Source>>,
    Source: Lookup<Pipeline<Source>>,
    Source: Lookup<PipelineSchedule<Source>>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<User<Source>>,
    Sink: Lookup<Environment<Sink>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<MergeRequest<Sink>>,
    Sink: Lookup<Pipeline<Sink>>,
    Sink: Lookup<PipelineSchedule<Sink>>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<User<Sink>>,
{
    environments: &'a IndexMap<Source, Sink, Environment<Source>, Environment<Sink>>,
    pipelines: &'a IndexMap<Source, Sink, Pipeline<Source>, Pipeline<Sink>>,
}

impl<'a, Source, Sink> Migration<Source, Sink, Deployment<Source>, Deployment<Sink>>
    for DeploymentMigration<'a, Source, Sink>
where
    Source: DiscoverableLookup<Deployment<Source>>,
    Source: Lookup<Environment<Source>>,
    Source: Lookup<Instance>,
    Source: Lookup<MergeRequest<Source>>,
    Source: Lookup<Pipeline<Source>>,
    Source: Lookup<PipelineSchedule<Source>>,
    Source: Lookup<Project<Source>>,
    Source: Lookup<User<Source>>,
    <Source as Lookup<Deployment<Source>>>::Index: Ord,
    <Source as Lookup<Environment<Source>>>::Index: Ord,
    <Source as Lookup<Pipeline<Source>>>::Index: Ord,
    Sink: DiscoverableLookup<Deployment<Sink>>,
    Sink: Lookup<Environment<Sink>>,
    Sink: Lookup<Instance>,
    Sink: Lookup<MergeRequest<Sink>>,
    Sink: Lookup<Pipeline<Sink>>,
    Sink: Lookup<PipelineSchedule<Sink>>,
    Sink: Lookup<Project<Sink>>,
    Sink: Lookup<User<Sink>>,
{
    fn migrate(
        &self,
        source: &Source,
        sink: &mut Sink,
        imap: &mut IndexMap<Source, Sink, Deployment<Source>, Deployment<Sink>>,
    ) -> Result<(), MigrationError> {
        for idx in source.all_indices() {
            let entry = imap.entry(idx)?;
            let data: Deployment<Source> = get_data(source, entry.key())?;

            // TODO: check if the sink already has this `Deployment`.

            let mut new_data: Deployment<Sink> = Deployment::builder()
                .pipeline(self.pipelines.get(&data.pipeline)?)
                .environment(self.environments.get(&data.environment)?)
                .forge_id(data.forge_id)
                .created_at(data.created_at)
                .updated_at(data.updated_at)
                .status(data.status)
                .build()
                .unwrap();
            new_data.finished_at = data.finished_at;
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
    Source: DiscoverableLookup<Deployment<Source>>,
    Source: DiscoverableLookup<Environment<Source>>,
    Source: DiscoverableLookup<Instance>,
    Source: DiscoverableLookup<MergeRequest<Source>>,
    Source: DiscoverableLookup<Pipeline<Source>>,
    Source: DiscoverableLookup<PipelineSchedule<Source>>,
    Source: DiscoverableLookup<Project<Source>>,
    Source: DiscoverableLookup<Runner<Source>>,
    Source: DiscoverableLookup<RunnerHost>,
    Source: DiscoverableLookup<User<Source>>,
    <Source as Lookup<Deployment<Source>>>::Index: Ord,
    <Source as Lookup<Environment<Source>>>::Index: Ord,
    <Source as Lookup<Instance>>::Index: Ord,
    <Source as Lookup<MergeRequest<Source>>>::Index: Ord,
    <Source as Lookup<Pipeline<Source>>>::Index: Ord,
    <Source as Lookup<PipelineSchedule<Source>>>::Index: Ord,
    <Source as Lookup<Project<Source>>>::Index: Ord,
    <Source as Lookup<Runner<Source>>>::Index: Ord,
    <Source as Lookup<RunnerHost>>::Index: Ord,
    <Source as Lookup<User<Source>>>::Index: Ord,
    Sink: DiscoverableLookup<Deployment<Sink>>,
    Sink: DiscoverableLookup<Environment<Sink>>,
    Sink: DiscoverableLookup<Instance>,
    Sink: DiscoverableLookup<MergeRequest<Sink>>,
    Sink: DiscoverableLookup<Pipeline<Sink>>,
    Sink: DiscoverableLookup<PipelineSchedule<Sink>>,
    Sink: DiscoverableLookup<Project<Sink>>,
    Sink: DiscoverableLookup<Runner<Sink>>,
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

    // Runners
    let mut runner_map = IndexMap::<Source, Sink, Runner<Source>, Runner<Sink>>::default();
    {
        let migration = RunnerMigration {
            instances: &mut instance_map,
            projects: &mut project_map,
            runner_hosts: &mut runner_host_map,
        };
        migration.migrate(source, sink, &mut runner_map)?;
    }

    // Merge requests
    let mut merge_request_map =
        IndexMap::<Source, Sink, MergeRequest<Source>, MergeRequest<Sink>>::default();
    {
        let migration = MergeRequestMigration {
            projects: &mut project_map,
            users: &mut user_map,
        };
        migration.migrate(source, sink, &mut merge_request_map)?;
    }

    // Pipeline schedules
    let mut pipeline_schedule_map =
        IndexMap::<Source, Sink, PipelineSchedule<Source>, PipelineSchedule<Sink>>::default();
    {
        let migration = PipelineScheduleMigration {
            projects: &mut project_map,
            users: &mut user_map,
        };
        migration.migrate(source, sink, &mut pipeline_schedule_map)?;
    }

    // Pipelines
    let mut pipeline_map = IndexMap::<Source, Sink, Pipeline<Source>, Pipeline<Sink>>::default();
    {
        let migration = PipelineMigration {
            projects: &mut project_map,
            pipeline_schedules: &mut pipeline_schedule_map,
            merge_requests: &mut merge_request_map,
            users: &mut user_map,
        };
        migration.migrate(source, sink, &mut pipeline_map)?;
    }

    // Environments
    let mut environment_map =
        IndexMap::<Source, Sink, Environment<Source>, Environment<Sink>>::default();
    {
        let migration = EnvironmentMigration {
            projects: &mut project_map,
        };
        migration.migrate(source, sink, &mut environment_map)?;
    }

    // Deployments
    let mut deployment_map =
        IndexMap::<Source, Sink, Deployment<Source>, Deployment<Sink>>::default();
    {
        let migration = DeploymentMigration {
            environments: &mut environment_map,
            pipelines: &mut pipeline_map,
        };
        migration.migrate(source, sink, &mut deployment_map)?;
    }

    // Jobs
    // Job artifacts

    Ok(())
}
