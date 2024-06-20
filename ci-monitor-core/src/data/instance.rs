// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// An instance of a forge which hosts projects.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Instance {
    /// A unique ID for the instance.
    pub unique_id: u64,
    /// The name of the forge implementation.
    pub forge: String,
    /// The URL of the forge.
    pub url: String,
}
