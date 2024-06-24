// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;
use std::sync::Arc;

use ci_monitor_core::Lookup;

/// A mechanism to use `Arc` instances to resolve themselves.
#[derive(Debug, Clone)]
pub struct ArcLookup;

/// The "index" of `ArcLookup`.
///
/// This is actually just the `Arc` itself.
pub struct ArcIndex<T> {
    arc: Arc<T>,
}

impl<T> ArcIndex<T> {
    /// Extract the `Arc` from the index.
    pub fn into_inner(self) -> Arc<T> {
        self.arc
    }
}

impl<T> From<Arc<T>> for ArcIndex<T> {
    fn from(arc: Arc<T>) -> Self {
        Self {
            arc,
        }
    }
}

impl<T> Clone for ArcIndex<T> {
    fn clone(&self) -> Self {
        Self {
            arc: self.arc.clone(),
        }
    }
}

impl<T> Debug for ArcIndex<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.arc)
    }
}

impl<T> Lookup<T> for ArcLookup
where
    T: Debug + Send + Sync,
{
    type Index = ArcIndex<T>;

    fn lookup<'a>(&'a self, idx: &'a Self::Index) -> Option<&'a T> {
        Some(&idx.arc)
    }

    fn store(&mut self, data: T) -> Self::Index {
        Self::Index {
            arc: Arc::new(data),
        }
    }
}
