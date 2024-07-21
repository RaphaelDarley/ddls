// pub static RUNNING:

use std::collections::HashMap;

use tokio::task::JoinSet;
use tracing::{error, info};

use crate::proc::{Proc, ProcDesc};

pub async fn manager_start<I>(procs: I)
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
}
