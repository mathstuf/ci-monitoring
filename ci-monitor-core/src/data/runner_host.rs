// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use derive_builder::Builder;

/// Information about a machine that performs jobs.
#[derive(Debug, Builder, Clone)]
#[builder(pattern = "owned")]
#[non_exhaustive]
pub struct RunnerHost {
    // Metadata.
    /// The operating system.
    #[builder(default, setter(into))]
    pub os: String,
    /// The operating system version.
    #[builder(default, setter(into))]
    pub os_version: String,
    /// The name of the host.
    ///
    /// Hostname or instance type name.
    #[builder(setter(into))]
    pub name: String,
    /// How the host is managed.
    #[builder(default, setter(into))]
    pub management: String,
    /// Where the host lives.
    #[builder(default, setter(into))]
    pub location: String,

    /// An estimate of the cost of tasks on this host per hour.
    #[builder(default)]
    pub estimated_cost_per_hour: Option<f64>,

    /// A unique ID for the runner host.
    pub unique_id: u64,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    #[builder(default = "Utc::now()", setter(skip))]
    pub cim_refreshed_at: DateTime<Utc>,
}

impl RunnerHost {
    /// Create a builder for the structure.
    pub fn builder() -> RunnerHostBuilder {
        RunnerHostBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{RunnerHost, RunnerHostBuilderError};

    #[test]
    fn name_is_required() {
        let err = RunnerHost::builder().unique_id(0).build().unwrap_err();
        crate::test::assert_missing_field!(err, RunnerHostBuilderError, "name");
    }

    #[test]
    fn unique_id_is_required() {
        let err = RunnerHost::builder().name("name").build().unwrap_err();
        crate::test::assert_missing_field!(err, RunnerHostBuilderError, "unique_id");
    }

    #[test]
    fn sufficient_fields() {
        RunnerHost::builder()
            .name("name")
            .unique_id(0)
            .build()
            .unwrap();
    }
}
