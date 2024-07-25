// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
/// Errors which can occur when storing or loading a `VecLookup` store.
pub enum VecStoreError {
    /// JSON error.
    #[error("JSON error: {}", source)]
    Json {
        /// The JSON error.
        #[from]
        source: serde_json::Error,
    },
    /// I/O error.
    #[error("i/o error: {}", source)]
    Io {
        /// The error.
        #[from]
        source: io::Error,
    },
}
