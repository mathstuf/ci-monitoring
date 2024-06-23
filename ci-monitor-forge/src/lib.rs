// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! CI monitoring forge support
//!
//! This crate defines the core functionality around forge communication for CI monitoring. Actual
//! implementations can `impl` the traits here and use the helpers to more easily support gathering
//! the required information.

#![warn(missing_docs)]
