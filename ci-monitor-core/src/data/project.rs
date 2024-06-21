// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};

use crate::data::Instance;
use crate::Lookup;

/// An instance of a project.
///
/// This represents an instance of a project. There may be multiple instances of the project on
/// different instances or even on a given instance.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Project<L>
where
    L: Lookup<Instance>,
{
    // Metadata.
    /// The name of the project.
    ///
    /// This is informal and a project may exist at multiple locations.
    pub name: String,

    // Forge metadata.
    /// The ID of the project.
    pub forge_id: u64,
    /// The URL of the project.
    pub url: String,
    /// The instance on which the project lives.
    pub instance: <L as Lookup<Instance>>::Index,
    /// The path to the repository on the instance.
    pub instance_path: String,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
