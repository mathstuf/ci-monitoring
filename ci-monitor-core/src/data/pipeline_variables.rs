// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::BTreeMap;

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
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PipelineVariable {
    /// The value of the pipeline variable.
    pub value: String,
    /// How the pipeline variable is made available to jobs.
    pub type_: PipelineVariableType,
    /// Whether the variable is protected or not.
    pub protected: bool,
    /// The environment the variable is made available to.
    pub environment: Option<String>,
}

/// A set of pipeline variables.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PipelineVariables {
    /// The variables.
    pub variables: BTreeMap<String, PipelineVariable>,
}
