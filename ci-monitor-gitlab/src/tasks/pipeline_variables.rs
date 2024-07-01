// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ci_monitor_core::data::{PipelineVariable, PipelineVariableType, PipelineVariables};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
enum GitlabVariableType {
    #[serde(rename = "env_var")]
    EnvironmentVariable,
    #[serde(rename = "file")]
    File,
}

impl From<GitlabVariableType> for PipelineVariableType {
    fn from(gvt: GitlabVariableType) -> Self {
        match gvt {
            GitlabVariableType::EnvironmentVariable => Self::String,
            GitlabVariableType::File => Self::File,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GitlabPipelineVariable {
    variable_type: GitlabVariableType,
    key: String,
    value: String,
}

/// Convert a set of GitLab variables to the monitoring representation.
pub fn gitlab_variables(gpvs: Vec<GitlabPipelineVariable>) -> PipelineVariables {
    gpvs.into_iter()
        .map(|gpv| {
            (
                gpv.key,
                PipelineVariable::builder()
                    .value(gpv.value)
                    .type_(gpv.variable_type.into())
                    .build()
                    .unwrap(),
            )
        })
        .collect()
}
