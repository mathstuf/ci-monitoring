// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{DateTime, Utc};

use crate::data::{BlobReference, Instance};
use crate::Lookup;

/// A user account on an instance.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct User<L>
where
    L: Lookup<Instance>,
{
    /// The handle of the user.
    pub handle: String,
    /// The display name of the user.
    pub name: String,
    /// The email address of the user.
    pub email: Option<String>,
    /// The avatar of the user.
    pub avatar: Option<BlobReference>,

    // Forge metadata.
    /// The ID of the user.
    pub forge_id: u64,
    /// The instance the user account is associated with.
    pub instance: <L as Lookup<Instance>>::Index,

    // Monitoring metadata.
    /// When the monitoring tool first fetched information.
    pub cim_fetched_at: DateTime<Utc>,
    /// When the monitoring tool last updated this information.
    pub cim_refreshed_at: DateTime<Utc>,
}
