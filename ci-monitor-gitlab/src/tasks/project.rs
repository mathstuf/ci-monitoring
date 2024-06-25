// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ci_monitor_core::data::{Instance, Project};
use ci_monitor_core::Lookup;
use ci_monitor_forge::{ForgeError, ForgeTaskOutcome};
use ci_monitor_persistence::DiscoverableLookup;

use crate::GitlabForge;

pub async fn update_project<L>(
    forge: &GitlabForge<L>,
    project: u64,
) -> Result<ForgeTaskOutcome, ForgeError>
where
    L: DiscoverableLookup<Project<L>>,
    L: Lookup<Instance>,
    L: Send + Sync,
{
    let mut outcome = ForgeTaskOutcome::default();
    let mut add_task = |task| outcome.additional_tasks.push(task);

    // TODO: fetch the project
    // TODO: queue MR discovery
    // TODO: queue pipeline discovery
    // TODO: queue environments discovery
    // TODO: parent project
    // TODO: update storage

    Ok(outcome)
}
