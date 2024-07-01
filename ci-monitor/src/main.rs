// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error::Error;

use clap::Command;

/// A `main` function which supports `try!`.
fn try_main() -> Result<(), Box<dyn Error>> {
    let _ = Command::new("ci-monitor")
        .version(clap::crate_version!())
        .author("Ben Boeckel <ben.boeckel@kitware.com>")
        .about("Monitor CI on a forge to store for further analysis")
        .get_matches();

    Ok(())
}

fn main() {
    if let Err(err) = try_main() {
        panic!("{:?}", err);
    }
}
