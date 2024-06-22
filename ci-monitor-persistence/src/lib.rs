// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! CI monitoring persistence
//!
//! Core traits and basic implementations of persistence for CI monitoring.

#![warn(missing_docs)]

mod blob;
mod objects;

pub use self::blob::BlobPersistence;
pub use self::blob::BlobPersistenceAsync;
pub use self::blob::BlobPersistenceError;
pub use self::blob::BlobPersistenceVerifyError;

pub use self::objects::ArcIndex;
pub use self::objects::ArcLookup;

pub use self::objects::VecIndex;
pub use self::objects::VecLookup;
