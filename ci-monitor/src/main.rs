// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::VecDeque;
use std::error::Error;

use ci_monitor_forge::{Forge, ForgeTask};
use ci_monitor_gitlab::gitlab;
use ci_monitor_gitlab::GitlabForge;
use ci_monitor_persistence::VecLookup;
use clap::{Arg, ArgAction, Command};

/// A `main` function which supports `try!`.
async fn try_main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("ci-monitor")
        .version(clap::crate_version!())
        .author("Ben Boeckel <ben.boeckel@kitware.com>")
        .about("Monitor CI on a forge to store for further analysis")
        .arg(
            Arg::new("TOKEN")
                .short('t')
                .long("token")
                .help("Token to use")
                .action(ArgAction::Set),
        )
        .get_matches();

    let token = matches.get_one::<String>("TOKEN").unwrap();
    let gitlab = gitlab::GitlabBuilder::new("gitlab.kitware.com", token)
        .build_async()
        .await
        .unwrap();
    let storage = VecLookup::default();
    let forge = GitlabForge::new("gitlab.kitware.com", gitlab, storage);

    let mut tasks: VecDeque<ForgeTask> = VecDeque::new();
    tasks.push_back(ForgeTask::DiscoverRunners {});
    tasks.push_back(ForgeTask::UpdateProject {
        project: 13,
    });

    let mut count = 0;
    while let Some(task) = tasks.pop_front() {
        println!(
            "performing task {} ({} remaining): {:?}",
            count,
            tasks.len(),
            task,
        );
        count += 1;
        let res = forge.run_task_async(task).await;
        match res {
            Ok(outcome) => {
                tasks.extend(outcome.additional_tasks);
            },
            Err(err) => {
                println!("failed: {:?}", err);
            },
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = try_main().await {
        panic!("{:?}", err);
    }
}
