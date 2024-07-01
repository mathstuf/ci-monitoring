// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod merge_request;
mod pipeline_schedule;
mod pipeline_variables;
mod project;
mod runner;
mod user;

pub use self::merge_request::discover_merge_requests;
pub use self::merge_request::update_merge_request;

pub use self::pipeline_schedule::discover_pipeline_schedules;
pub use self::pipeline_schedule::update_pipeline_schedule;

use self::pipeline_variables::gitlab_variables;
use self::pipeline_variables::GitlabPipelineVariable;

pub use self::project::update_project;
pub use self::project::update_project_by_name;

pub use self::runner::discover_runners;
pub use self::runner::update_runner;

pub use self::user::update_user;
pub use self::user::update_user_by_name;
