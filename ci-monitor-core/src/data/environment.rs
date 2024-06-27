// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};
use perfect_derive::perfect_derive;

use crate::data::{Instance, Project};
use crate::Lookup;

/// The state of an environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvironmentState {
    /// The environment is available.
    Available,
    /// The environment is shutting down.
    Stopping,
    /// The environment is stopped.
    Stopped,
}

/// The environment tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvironmentTier {
    /// An environment intended for production.
    Production,
    /// An environment for staging before production.
    Staging,
    /// An environment for testing.
    Testing,
    /// An environment for development.
    Development,
    /// An environment for other purposes.
    Other,
}

/// An environment into which deployments may be made.
#[perfect_derive(Debug, Clone)]
#[non_exhaustive]
pub struct Environment<L>
where
    L: Lookup<Instance>,
    L: Lookup<Project<L>>,
{
    // Metadata.
    /// The name of the environment.
    pub name: String,
    /// The external URL of the environment.
    pub external_url: String,
    /// The state of the environment.
    pub state: EnvironmentState,
    /// The tier of the environment.
    pub tier: EnvironmentTier,

    // Forge metadata.
    /// The ID of the environment.
    pub forge_id: u64,
    /// The project the environment is for.
    pub project: <L as Lookup<Project<L>>>::Index,
    /// When the environment was created.
    pub created_at: DateTime<Utc>,
    /// When the environment was updated.
    pub updated_at: DateTime<Utc>,
    /// When the environment will automatically stop.
    pub auto_stop_at: Option<DateTime<Utc>>,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
