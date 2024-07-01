// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;
use std::marker::PhantomData;

use ci_monitor_core::data::{
    Deployment, Environment, Instance, Job, JobArtifact, MergeRequest, Pipeline, PipelineSchedule,
    Project, Runner, RunnerHost, User,
};
use ci_monitor_core::Lookup;
use perfect_derive::perfect_derive;

use crate::DiscoverableLookup;

/// Storage for CI monitoring data backed by `Vec`.
///
/// Intended only for in-memory storage; no actual persistence is offered as removing data is
/// infeasible due to having to rewrite all indices to account for holes.
#[derive(Clone)]
pub struct VecLookup {
    deployments: Vec<Deployment<Self>>,
    environments: Vec<Environment<Self>>,
    instances: Vec<Instance>,
    jobs: Vec<Job<Self>>,
    job_artifacts: Vec<JobArtifact<Self>>,
    merge_requests: Vec<MergeRequest<Self>>,
    pipelines: Vec<Pipeline<Self>>,
    pipeline_schedules: Vec<PipelineSchedule<Self>>,
    projects: Vec<Project<Self>>,
    runners: Vec<Runner<Self>>,
    runner_hosts: Vec<RunnerHost>,
    users: Vec<User<Self>>,
}

impl Debug for VecLookup {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("VecLookup")
            .field("#deployments", &self.deployments.len())
            .field("#environments", &self.environments.len())
            .field("#instances", &self.instances.len())
            .field("#jobs", &self.jobs.len())
            .field("#job_artifacts", &self.job_artifacts.len())
            .field("#merge_requests", &self.merge_requests.len())
            .field("#pipelines", &self.pipelines.len())
            .field("#pipeline_schedules", &self.pipeline_schedules.len())
            .field("#projects", &self.projects.len())
            .field("#runners", &self.runners.len())
            .field("#runner_hosts", &self.runner_hosts.len())
            .field("#users", &self.users.len())
            .finish()
    }
}

/// The index of `VecLookup`.
#[perfect_derive(Debug, Copy, PartialEq, Eq)]
pub struct VecIndex<T> {
    idx: usize,
    _phantom: PhantomData<T>,
}

impl<T> Clone for VecIndex<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> VecIndex<T> {
    fn new(idx: usize) -> Self {
        Self {
            idx,
            _phantom: PhantomData,
        }
    }
}

trait HasId {
    fn id(&self) -> u64;
    fn has_id(&self, id: u64) -> bool;
}

macro_rules! impl_has_id_by {
    ($t:ty, $field:ident) => {
        impl HasId for $t {
            #[allow(clippy::misnamed_getters)]
            fn id(&self) -> u64 {
                self.$field
            }

            fn has_id(&self, id: u64) -> bool {
                self.$field == id
            }
        }
    };
}

impl_has_id_by!(Deployment<VecLookup>, forge_id);
impl_has_id_by!(Environment<VecLookup>, forge_id);
impl_has_id_by!(Instance, unique_id);
impl_has_id_by!(Job<VecLookup>, forge_id);
impl_has_id_by!(JobArtifact<VecLookup>, unique_id);
impl_has_id_by!(MergeRequest<VecLookup>, forge_id);
impl_has_id_by!(Pipeline<VecLookup>, forge_id);
impl_has_id_by!(PipelineSchedule<VecLookup>, forge_id);
impl_has_id_by!(Project<VecLookup>, forge_id);
impl_has_id_by!(Runner<VecLookup>, forge_id);
impl_has_id_by!(RunnerHost, unique_id);
impl_has_id_by!(User<VecLookup>, forge_id);

macro_rules! impl_lookup {
    ($t:ty, $field:ident) => {
        impl Lookup<$t> for VecLookup {
            type Index = VecIndex<$t>;

            fn lookup<'a>(&'a self, idx: &'a Self::Index) -> Option<&'a $t> {
                self.$field.get(idx.idx)
            }

            fn store(&mut self, data: $t) -> Self::Index {
                if let Some((idx, entry)) = self
                    .$field
                    .iter_mut()
                    .enumerate()
                    .find(|(_, e)| e.has_id(data.id()))
                {
                    *entry = data;
                    Self::Index::new(idx)
                } else {
                    let idx = self.$field.len();
                    self.$field.push(data);
                    Self::Index::new(idx.into())
                }
            }
        }

        impl DiscoverableLookup<$t> for VecLookup {
            fn all_indices(&self) -> Vec<Self::Index> {
                (0..self.$field.len()).map(Self::Index::new).collect()
            }

            fn find(&self, id: u64) -> Option<Self::Index> {
                self.$field
                    .iter()
                    .enumerate()
                    .find(|(_, ent)| ent.has_id(id))
                    .map(|(idx, _)| Self::Index::new(idx))
            }
        }
    };
}

impl_lookup!(Deployment<Self>, deployments);
impl_lookup!(Environment<Self>, environments);
impl_lookup!(Instance, instances);
impl_lookup!(Job<Self>, jobs);
impl_lookup!(JobArtifact<Self>, job_artifacts);
impl_lookup!(MergeRequest<Self>, merge_requests);
impl_lookup!(Pipeline<Self>, pipelines);
impl_lookup!(PipelineSchedule<Self>, pipeline_schedules);
impl_lookup!(Project<Self>, projects);
impl_lookup!(Runner<Self>, runners);
impl_lookup!(RunnerHost, runner_hosts);
impl_lookup!(User<Self>, users);
