// pub static RUNNING:

use std::collections::HashMap;

use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use tokio::{sync::mpsc, task::JoinSet};
use tracing::{error, info};

use crate::{
    message::Message,
    proc::{Proc, ProcDesc},
};

pub async fn manager_start<I>(procs: I, mut manager_rx: mpsc::Receiver<Message>)
where
    I: IntoIterator<Item = ProcDesc>,
    // I::Item = (String, ProcDesc),
{
    info!("manager started");
    let mut running: HashMap<String, Proc> = HashMap::new();
    let mut start_set = JoinSet::new();

    for d in procs {
        info!(proc_name = d.name, "adding to start_set");
        start_set.spawn(async move { d.exec_to_run_cached().await });
    }

    while let Some(res) = start_set.join_next().await {
        match res {
            Ok(r) => match r {
                Ok(p) => {
                    running.insert(p.desc.name.clone(), p);
                }
                Err((_p, _e)) => {}
            },
            Err(e) => {
                error!(err = ?e, "error joining proc start");
                continue;
            }
        };
    }

    while let Some(msg) = manager_rx.recv().await {
        info!(?msg, "manager recieved message");
        match msg {
            Message::UpdateSingleProc { name, ack } => {
                let mut proc = running.remove(&name).unwrap();
                proc.desc.update().await.unwrap();
                proc.desc.prepare().await.unwrap();

                if let Err(_e) = signal::kill(
                    Pid::from_raw(proc.child.id().unwrap() as i32),
                    Signal::SIGTERM,
                ) {
                    proc.child.kill().await.expect("kill should work")
                }

                let new_proc = proc.desc.run().await.expect("running should succeed");
                running.insert(new_proc.desc.name.clone(), new_proc);
                ack.send(Ok(())).expect("ack send should work");
            }
        }
    }
}
