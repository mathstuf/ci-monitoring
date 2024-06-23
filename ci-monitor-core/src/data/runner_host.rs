// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};

/// Information about a machine that performs jobs.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RunnerHost {
    // Metadata.
    /// The operating system.
    pub os: String,
    /// The operating system version.
    pub os_version: String,
    /// The name of the host.
    ///
    /// Hostname or instance type name.
    pub name: String,
    /// How the host is managed.
    pub management: String,
    /// Where the host lives.
    pub location: String,

    /// An estimate of the cost of tasks on this host per hour.
    pub estimated_cost_per_hour: Option<f64>,

    /// A unique ID for the runner host.
    pub unique_id: u64,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
