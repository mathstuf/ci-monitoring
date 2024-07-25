// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::{Deserialize, Serialize};
use std::any;
use std::fmt::Debug;

use super::VecStoreError;

fn invalid_enum_string<T>(value: &str) -> VecStoreError {
    VecStoreError::InvalidEnumString {
        typename: std::any::type_name::<T>(),
        value: value.into(),
    }
}

fn enum_to_string_opt<T>(lut: &[(T, &'static str)], en: T) -> Option<&'static str>
where
    T: Debug,
    T: PartialEq<T>,
{
    for (e, s) in lut {
        if *e == en {
            return Some(s);
        }
    }

    None
}

fn enum_to_string<T>(lut: &[(T, &'static str)], en: T) -> &'static str
where
    T: Copy + Debug,
    T: PartialEq<T>,
{
    if let Some(s) = enum_to_string_opt(lut, en) {
        s
    } else {
        panic!(
            "unexpected enum value for {}: {:?}",
            any::type_name::<T>(),
            en,
        );
    }
}

fn enum_from_string<T>(lut: &[(T, &'static str)], st: &str) -> Result<T, VecStoreError>
where
    T: Copy,
    T: PartialEq<T>,
{
    for (e, s) in lut {
        if *s == st {
            return Ok(*e);
        }
    }

    Err(invalid_enum_string::<T>(st))
}

pub(super) trait JsonConvert<T>: for<'a> Deserialize<'a> + Serialize {
    fn convert_to_json(o: &T) -> Self;
    fn create_from_json(&self) -> Result<T, VecStoreError>;
}
