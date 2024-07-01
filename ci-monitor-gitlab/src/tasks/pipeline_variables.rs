// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
enum GitlabVariableType {
    #[serde(rename = "env_var")]
    EnvironmentVariable,
    #[serde(rename = "file")]
    File,
}

#[derive(Debug, Deserialize)]
pub struct GitlabPipelineVariable {
    variable_type: GitlabVariableType,
    key: String,
    value: String,
}
