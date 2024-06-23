// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::time::Duration;

use ci_monitor_core::data::Instance;

/// A trait describing basic `Forge` capabilities.
pub trait ForgeCore {
    /// Obtain the `Instance` description for the forge.
    fn instance(&self) -> Instance;
}
