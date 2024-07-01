// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;

/// A trait to lookup other data based on an index.
pub trait Lookup<T> {
    /// The type used to lookup instances of `T`.
    type Index: Debug + Clone + Send + Sync;

    /// Find an instance of `T` given an index.
    fn lookup<'a>(&'a self, idx: &'a Self::Index) -> Option<&'a T>;
    /// Store an instance of `T` returning an index to get it again.
    fn store(&mut self, data: T) -> Self::Index;
}
