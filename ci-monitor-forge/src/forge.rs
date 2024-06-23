// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::time::Duration;

use async_trait::async_trait;
use ci_monitor_core::data::Instance;
use thiserror::Error;

use crate::ForgeTask;

/// The outcome of a forge task.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct ForgeTaskOutcome {
    /// Additonal tasks that were discovered during the task.
    pub additional_tasks: Vec<ForgeTask>,
    /// How long to delay the given tasks.
    ///
    /// Maybe used to avoid API rate limits.
    pub task_delay: Option<Duration>,
}

/// An error that may occur when performing a task.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ForgeError {
    /// Authentication failed.
    #[error("cannot authenticate to the forge: {}", details)]
    Auth {
        /// Details of the error.
        details: String,
    },
    /// The connection to the forge failed.
    #[error("cannot contact the forge: {}", details)]
    Connection {
        /// Details of the error.
        details: String,
    },
    /// The forge does not handle the specified task.
    #[error("task is not handled")]
    Unhandled {
        /// The unhandled task.
        task: ForgeTask,
    },
    /// The forge does not know about the specified task.
    #[error("task is not known")]
    Unknown {
        /// The unknown task.
        task: ForgeTask,
    },
    /// An uncategorized error.
    #[error("{}", details)]
    Other {
        /// Details of the error.
        details: String,
    },
}

/// A trait describing basic `Forge` capabilities.
pub trait ForgeCore {
    /// Obtain the `Instance` description for the forge.
    fn instance(&self) -> Instance;
}

/// A trait describing basic `Forge` capabilities.
#[async_trait]
pub trait Forge {
    /// Run a task.
    async fn run_task_async(&self, task: ForgeTask) -> Result<ForgeTaskOutcome, ForgeError>;
}
