// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use async_trait::async_trait;
use ci_monitor_core::data::Instance;
use ci_monitor_forge::{Forge, ForgeCore, ForgeError, ForgeTask, ForgeTaskOutcome};

/// A CI monitoring task handler for GitLab hosts.
pub struct GitlabForge {
    url: String,
}

impl ForgeCore for GitlabForge {
    fn instance(&self) -> Instance {
        Instance::builder()
            .forge("gitlab")
            .url(self.url.clone())
            .build()
            .unwrap()
    }
}

#[async_trait]
impl Forge for GitlabForge {
    /// Run a task.
    async fn run_task_async(&self, task: ForgeTask) -> Result<ForgeTaskOutcome, ForgeError> {
        match task {
            _ => {
                Err(ForgeError::Unknown {
                    task,
                })
            },
        }
    }
}
