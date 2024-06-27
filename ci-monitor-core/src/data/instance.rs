// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

/// An instance of a forge which hosts projects.
#[derive(Debug, Builder, Clone)]
#[builder(pattern = "owned")]
#[non_exhaustive]
pub struct Instance {
    /// A unique ID for the instance.
    pub unique_id: u64,
    /// The name of the forge implementation.
    #[builder(setter(into))]
    pub forge: String,
    /// The URL of the forge.
    #[builder(setter(into))]
    pub url: String,
}

impl Instance {
    /// Create a builder for the structure.
    pub fn builder() -> InstanceBuilder {
        InstanceBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{Instance, InstanceBuilderError};

    #[test]
    fn unique_id_is_required() {
        let err = Instance::builder()
            .forge("forge")
            .url("url")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, InstanceBuilderError, "unique_id");
    }

    #[test]
    fn forge_is_required() {
        let err = Instance::builder()
            .unique_id(0)
            .url("url")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, InstanceBuilderError, "forge");
    }

    #[test]
    fn url_is_required() {
        let err = Instance::builder()
            .unique_id(0)
            .forge("forge")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, InstanceBuilderError, "url");
    }

    #[test]
    fn sufficient_fields() {
        Instance::builder()
            .unique_id(0)
            .forge("forge")
            .url("url")
            .build()
            .unwrap();
    }
}
