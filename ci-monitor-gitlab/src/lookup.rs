// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ci_monitor_core::data::{
    Deployment, Environment, Instance, Job, JobArtifact, MergeRequest, Pipeline, PipelineSchedule,
    Project, Runner, RunnerHost, User,
};
use ci_monitor_core::Lookup;
use ci_monitor_persistence::{DiscoverableLookup, VecLookup};

pub trait GitlabLookup<L>:
    Lookup<Deployment<L>>
    + Lookup<Environment<L>>
    + Lookup<Job<L>>
    + Lookup<JobArtifact<L>>
    + Lookup<MergeRequest<L>>
    + Lookup<Pipeline<L>>
    + Lookup<PipelineSchedule<L>>
    + DiscoverableLookup<Project<L>>
    + DiscoverableLookup<Runner<L>>
    + DiscoverableLookup<RunnerHost>
    + DiscoverableLookup<User<L>>
    + DiscoverableLookup<Instance>
where
    L: Lookup<Deployment<L>>,
    L: Lookup<Environment<L>>,
    L: Lookup<Job<L>>,
    L: Lookup<MergeRequest<L>>,
    L: Lookup<Pipeline<L>>,
    L: Lookup<PipelineSchedule<L>>,
    L: Lookup<Project<L>>,
    L: Lookup<Runner<L>>,
    L: Lookup<RunnerHost>,
    L: Lookup<User<L>>,
    L: Lookup<Instance>,
{
}

impl GitlabLookup<Self> for VecLookup {}
