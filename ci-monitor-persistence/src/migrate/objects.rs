// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationError {}

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
