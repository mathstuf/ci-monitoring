// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! CI monitor core
//!
//! This crate defines core data types and traits for working with CI systems in order to monitor
//! them for overall health.

#![warn(missing_docs)]

pub mod data;
mod lookup;

pub use self::lookup::Lookup;
