// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ci_monitor_core::Lookup;

/// A `Lookup` that can also list what it contains.
pub trait DiscoverableLookup<T>: Lookup<T> {
    /// Return all indices.
    fn all_indices(&self) -> Vec<Self::Index>;
    /// Find an object by its ID.
    fn find(&self, id: u64) -> Option<Self::Index>;
}
