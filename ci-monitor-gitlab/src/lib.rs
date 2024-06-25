// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! CI monitoring for GitLab
//!
//! This crate provides CI monitoring with GitLab as a source of data.

#![warn(missing_docs)]

mod errors;
mod forge;
mod lookup;

pub use forge::GitlabForge;

use lookup::GitlabLookup;
