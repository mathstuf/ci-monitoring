// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error::Error;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use ci_monitor_forge::{Forge, ForgeTask};
use ci_monitor_gitlab::gitlab;
use ci_monitor_gitlab::GitlabForge;
use ci_monitor_persistence::VecLookup;
use clap::{Arg, ArgAction, Command};
use governor::{Jitter, Quota, RateLimiter};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

const QUEUE_SIZE: usize = 50;

async fn handle_tasks(
    forge: Arc<GitlabForge<VecLookup>>,
    send: UnboundedSender<ForgeTask>,
    mut recv: UnboundedReceiver<ForgeTask>,
) {
    let mut tokio_tasks = Vec::new();
    let mut count = 0;
    let governor = RateLimiter::direct(Quota::per_second(NonZeroU32::new(50).unwrap()));
    let jitter = Jitter::up_to(Duration::from_secs(2));

    while let Some(task) = recv.recv().await {
        governor.until_ready_with_jitter(jitter).await;

        println!(
            "performing task {} ({} remaining): {:?}",
            count,
            recv.len(),
            task,
        );
        count += 1;

        let inner_forge = forge.clone();
        let inner_send = send.clone();
        let async_task = tokio::spawn(async move {
            let res = inner_forge.run_task_async(task).await;
            match res {
                Ok(outcome) => {
                    for task in outcome.additional_tasks {
                        inner_send.send(task).unwrap();
                    }
                },
                Err(err) => {
                    println!("failed: {:?}", err);
                },
            }
        });

        tokio_tasks.push(async_task);

        if tokio_tasks.len() > QUEUE_SIZE {
            for tokio_task in tokio_tasks.drain(..QUEUE_SIZE) {
                tokio_task.await.unwrap();
            }
        }
    }

    for tokio_task in tokio_tasks {
        tokio_task.await.unwrap();
    }
}

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
    let forge = Arc::new(forge);

    let (send, recv) = tokio::sync::mpsc::unbounded_channel();
    send.send(ForgeTask::DiscoverRunners {}).unwrap();
    send.send(ForgeTask::UpdateProject {
        project: 13,
    })
    .unwrap();

    handle_tasks(forge, send, recv).await;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = try_main().await {
        panic!("{:?}", err);
    }
}
