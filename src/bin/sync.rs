
use std::time::Duration;

use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

use copperminds::{backup, vault};
use tokio::{select, time};

macro_rules! continue_on_err {
    ($event:expr, $msg:literal) => {
        match $event {
            Ok (e) => e,
            Err(e) => {
                eprintln!("{}: {e}", $msg);
                continue;
            },
        }
    };
}

#[tokio::main]
/// the purpose of this is to run a "sync" server that watches for changes on the main vault
/// and auto creates commits, and pushes the results to the backup git repo
async fn main() {

    let folder = vault::vault_folder();

    let (mut watcher, mut rx) = async_watcher().expect("Unable to get watcher");

    watcher
        .watch(folder.as_ref(), RecursiveMode::Recursive)
        .expect("Unable to add watch folder")
    ;

    println!("looping");
    while let Some(event) = next_write(&mut rx).await {
        println!("received write: {event:?}");

        wait_for_write_timeout(&mut rx, Duration::from_secs(10)).await;

        println!("making backup");
        backup::backup_named(&folder, "Automatic backup");
    }
}

// https://github.com/notify-rs/notify/blob/main/examples/async_monitor.rs
fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn next_write(rx: &mut Receiver<notify::Result<Event>>) -> Option<Event> {

    type Ek = notify::EventKind;

    while let Some(event) = rx.next().await {
        let event = continue_on_err!(event, "Error while reading event stream");

        let is_git = event
            .paths
            .iter()
            .any(|p| p
                .ancestors()
                .any(|p| p.is_dir() && p.ends_with(".git"))
            )
        ;

        if is_git {
            continue;
        }

        match event.kind {
              Ek::Create(_)
            | Ek::Modify(_)
            | Ek::Remove(_) => {},

            _ => continue,
        }

        return Some(event);
    };

    None

}

async fn wait_for_write_timeout(rx: &mut Receiver<notify::Result<Event>>, timeout: Duration) {

    loop {
        let sleep = time::sleep(timeout);
        tokio::pin!(sleep);

        select! {
            _ = &mut sleep => {
                break;
            }
            e = next_write(rx) => {
                println!("> new write: {e:?}")
            }
        }
    }
}
