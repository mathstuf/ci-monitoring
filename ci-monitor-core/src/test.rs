// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;
use std::sync::Arc;

use crate::Lookup;

macro_rules! assert_missing_field {
    ($err:expr, $type:tt, $field:expr $(,)?) => {
        let in_err = $err;
        let in_field = $field;
        if let $type::UninitializedField(field) = in_err {
            assert_eq!(field, in_field);
        } else {
            panic!(
                "unexpected error (expected to be missing `{}`): {:?}",
                in_field, in_err,
            );
        }
    };
}
pub(crate) use assert_missing_field;

#[derive(Debug, Default, Clone)]
pub struct TestLookup {}

/// The "index" of `ArcLookup`.
///
/// This is actually just the `Arc` itself.
pub struct TestIndex<T> {
    arc: Arc<T>,
}

impl<T> Clone for TestIndex<T> {
    fn clone(&self) -> Self {
        Self {
            arc: self.arc.clone(),
        }
    }
}

impl<T> Debug for TestIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "index for {}", std::any::type_name::<T>())
    }
}

impl<T> Lookup<T> for TestLookup
where
    T: Send + Sync,
{
    /// The type used to lookup instances of `T`.
    type Index = TestIndex<T>;

    /// Find an instance of `T` given an index.
    fn lookup<'a>(&'a self, idx: &'a Self::Index) -> Option<&'a T> {
        Some(&idx.arc)
    }
    /// Store an instance of `T` returning an index to get it again.
    fn store(&mut self, data: T) -> Self::Index {
        Self::Index {
            arc: Arc::new(data),
        }
    }
}
