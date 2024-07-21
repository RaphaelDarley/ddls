use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde::Deserialize;
use tokio::fs::{DirBuilder, File, OpenOptions};
use tokio::process::{Child, Command};
use tracing::{info, warn};

use crate::error::DdlsError;

/// Description of a process from config
#[derive(Debug, Clone, Deserialize)]
pub struct ProcDesc {
    pub name: String,
    pub dir: PathBuf,
    pub setup: String,
    pub prepare: String,
    pub run: String,
    pub update: String,
}

impl ProcDesc {
    // TODO: write to ddls_info.toml after each operation

    pub async fn exec_to_run_cached(self) -> Result<Proc, (ProcDesc, DdlsError)> {
        info!(proc_name = self.name, "exec started");
        if let Err(e) = self.setup().await {
            warn!(proc_name = self.name, err = ?e, "setup failed");
            return Err((self, e));
        }
        info!(proc_name = self.name, "setup succeeded");
        if let Err(e) = self.prepare().await {
            warn!(proc_name = self.name, err = ?e, "prepare failed");
            return Err((self, e));
        }
        info!(proc_name = self.name, "prepare succeeded");
        let out = self.run().await;
        match &out {
            Ok(p) => {
                info!(proc_name = p.desc.name, pid = p.child.id(), "run succeeded");
            }
            Err(e) => {
                warn!(proc_name = e.0.name, err = ?e.1, "run failed");
            }
        }
        out
    }

    pub async fn setup(&self) -> Result<(), DdlsError> {
        if !self.dir.exists() {
            DirBuilder::new()
                .recursive(true)
                .create(&self.dir)
                .await
                .expect("dir creation should succeed");
        }
        self.exec_ignore_out(&self.setup).await
    }

    pub async fn prepare(&self) -> Result<(), DdlsError> {
        self.exec_ignore_out(&self.prepare).await
    }

    pub async fn update(&self) -> Result<(), DdlsError> {
        self.exec_ignore_out(&self.prepare).await
    }

    pub async fn run(self) -> Result<Proc, (ProcDesc, DdlsError)> {
        // let log_file = OpenOptions::new()
        //     .append(true)
        //     .open(self.dir.join("ddls.log"))
        //     .await
        //     .expect("should be able to open log file");
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join("ddls.log"))
            .expect("should be able to open log file");
        let res = Command::new("sh")
            .arg("-c")
            .arg(&self.run)
            .current_dir(&self.dir)
            .stdout(
                log_file
                    .try_clone()
                    .expect("unused file should be cloneable"),
            )
            .stderr(log_file)
            .spawn()
            .map_err(|e| DdlsError::CmdInvoke {
                cmd: self.run.to_owned(),
                err: e.to_string(),
            });

        let child = match res {
            Ok(c) => c,
            Err(e) => return Err((self, e)),
        };

        let proc = Proc { child, desc: self };
        Ok(proc)
    }

    async fn exec_ignore_out(&self, cmd: &str) -> Result<(), DdlsError> {
        let out = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .current_dir(&self.dir)
            .stdout(Stdio::null())
            .output()
            .await
            .map_err(|e| DdlsError::CmdInvoke {
                cmd: cmd.to_owned(),
                err: e.to_string(),
            })?;

        if !out.status.success() {
            let stderr = match String::from_utf8(out.stderr) {
                Ok(s) => s,
                Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
            };
            return Err(DdlsError::CmdRun {
                cmd: cmd.to_owned(),
                exit: out.status.code(),
                stderr,
            });
        }

        Ok(())
    }
    pub fn log_file_path(&self) -> PathBuf {
        self.dir.join(format!("{}.log", self.name))
    }
    // pub async fn start(self, name: String) {
    //     let dir_path = Path::new("proc").join(name);
    //     if !dir_path.exists() {
    //         // DirBuilder::new()
    //         //     .recursive(true)
    //         //     .create(dir_path)
    //         //     .await
    //         //     .expect("dir creation should succeed");
    //         let dir_path_c = dir_path.clone();
    //         let url = self.git_url.clone();
    //         let res = spawn_blocking(move || async move { Repository::clone(&url, dir_path_c) })
    //             .await
    //             .unwrap()
    //             .await
    //             .expect("cloning should succeed");
    //     }
    // }
}

/// A running proccess
pub struct Proc {
    pub child: Child,
    pub desc: ProcDesc,
}
