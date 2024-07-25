// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::json::JsonConvert;
use super::{VecIndex, VecLookup, VecStoreError};

pub(super) trait JsonStorable: Sized {
    type Json: JsonConvert<Self>;

    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        let json = Self::Json::convert_to_json(self);
        serde_json::to_value(json)
    }

    fn from_json(json: serde_json::Value) -> Result<Self, VecStoreError> {
        let value: Self::Json = serde_json::from_value(json)?;
        value.create_from_json()
    }

    fn validate_indices(
        &self,
        self_index: VecIndex<Self>,
        storage: &VecLookup,
    ) -> Result<(), VecStoreError> {
        let _ = self_index;
        let _ = storage;
        Ok(())
    }
}
