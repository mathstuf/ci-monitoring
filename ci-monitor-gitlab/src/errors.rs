// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ci_monitor_forge::ForgeError;
use gitlab::api::ApiError;
use gitlab::RestError;

pub fn forge_error(err: ApiError<RestError>) -> ForgeError {
    let details = format!("{}", err);
    match err {
        ApiError::Auth {
            ..
        } => {
            ForgeError::Auth {
                details,
            }
        },
        ApiError::GitlabService {
            status, ..
        } => {
            if status.is_server_error() {
                ForgeError::Connection {
                    details,
                }
            } else if status.is_client_error() {
                ForgeError::Auth {
                    details,
                }
            } else {
                ForgeError::Other {
                    details,
                }
            }
        },
        _ => {
            ForgeError::Other {
                details,
            }
        },
    }
}
