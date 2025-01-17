// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::BTreeMap;

use derive_builder::Builder;

/// How the pipeline variable is available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PipelineVariableType {
    /// The value is placed as contents within a file.
    ///
    /// The environment variable contains the path to the file.
    File,
    /// The environment variable contains the contents of the variable.
    String,
}

/// A pipeline variable value.
#[derive(Debug, Builder, Clone)]
#[builder(pattern = "owned")]
#[non_exhaustive]
pub struct PipelineVariable {
    /// The value of the pipeline variable.
    #[builder(setter(into))]
    pub value: String,
    /// How the pipeline variable is made available to jobs.
    pub type_: PipelineVariableType,
    /// Whether the variable is protected or not.
    #[builder(default)]
    pub protected: bool,
    /// The environment the variable is made available to.
    #[builder(default)]
    pub environment: Option<String>,
}

impl PipelineVariable {
    /// Create a builder for the structure.
    pub fn builder() -> PipelineVariableBuilder {
        PipelineVariableBuilder::default()
    }
}

/// A set of pipeline variables.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PipelineVariables {
    /// The variables.
    pub variables: BTreeMap<String, PipelineVariable>,
}

impl FromIterator<(String, PipelineVariable)> for PipelineVariables {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, PipelineVariable)>,
    {
        Self {
            variables: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{PipelineVariable, PipelineVariableBuilderError, PipelineVariableType};

    #[test]
    fn value_is_required() {
        let err = PipelineVariable::builder()
            .type_(PipelineVariableType::File)
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineVariableBuilderError, "value");
    }

    #[test]
    fn type_is_required() {
        let err = PipelineVariable::builder()
            .value("value")
            .build()
            .unwrap_err();
        crate::test::assert_missing_field!(err, PipelineVariableBuilderError, "type_");
    }

    #[test]
    fn sufficient_fields() {
        PipelineVariable::builder()
            .value("value")
            .type_(PipelineVariableType::File)
            .build()
            .unwrap();
    }
}
