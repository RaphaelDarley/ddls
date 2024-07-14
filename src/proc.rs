use std::path::Path;

use git2::Repository;
use serde::Deserialize;
use tokio::{
    fs::{DirBuilder, File},
    task::spawn_blocking,
};
use tokio_process::Child;

/// Description of a process from config
#[derive(Debug, Deserialize)]
pub struct ProcDesc {
    setup_script: Option<String>,
    run_cmd: String,
    run_args: Vec<String>,
    git_url: String,
}

impl ProcDesc {
    pub async fn start(self, name: String) {
        let dir_path = Path::new("proc").join(name);
        if !dir_path.exists() {
            // DirBuilder::new()
            //     .recursive(true)
            //     .create(dir_path)
            //     .await
            //     .expect("dir creation should succeed");
            let dir_path_c = dir_path.clone();
            let url = self.git_url.clone();
            let res = spawn_blocking(move || async move { Repository::clone(&url, dir_path_c) })
                .await
                .unwrap()
                .await
                .expect("cloning should succeed");
        }
    }
}

/// A running proccess
pub struct Proc {
    child: Child,
    desc: ProcDesc,
}
