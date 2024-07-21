use axum::http::status;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::{read_to_string, write, File, OpenOptions};
use toml_edit::{de::from_document, ser::to_document, DocumentMut};
use tracing::{error, warn};

use crate::{error::DdlsError, proc::ProcDesc};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProcStatus {
    setup: Option<CmdStatus>,
    prepare: Option<CmdStatus>,
    run: Option<CmdStatus>,
    update: Option<CmdStatus>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CmdStatus {
    executed: String,
    time: NaiveDateTime,
}

pub enum Cmd {
    Setup,
    Prepare,
    Run,
    Update,
}

// impl Cmd {
//     fn to_str(&self) -> &'static str {
//         match self {
//             Cmd::Setup => "setup",
//             Cmd::Prepare => "prepare",
//             Cmd::Run => "run",
//             Cmd::Update => "update",
//         }
//     }
// }

pub async fn status_is_done(desc: &ProcDesc, cmd_type: Cmd) -> Result<bool, DdlsError> {
    let status = read_status(desc).await?;
    let cmd_status = match cmd_type {
        Cmd::Setup => status.setup,
        Cmd::Prepare => status.prepare,
        Cmd::Run => status.run,
        Cmd::Update => status.update,
    };

    let Some(prev) = cmd_status else {
        return Ok(false);
    };

    let next_cmd = match cmd_type {
        Cmd::Setup => &desc.setup,
        Cmd::Prepare => &desc.setup,
        Cmd::Run => &desc.setup,
        Cmd::Update => &desc.setup,
    };
    Ok(prev.executed == *next_cmd)
}

pub async fn read_status(desc: &ProcDesc) -> Result<ProcStatus, DdlsError> {
    let status_path = desc.dir.join("status.toml");

    let status_str = read_to_string(&status_path).await.unwrap_or_default();

    // let status_str = read_to_string(&status_path)
    //     .await
    //     .inspect_err(|e| warn!(proc_name = desc.name, err = ?e, "error updating status"))?;

    let doc = status_str.parse::<DocumentMut>().unwrap_or_else(|e| {
        error!(proc_name = desc.name, err = ?e, "status files are auto generated so should always be valid");
        DocumentMut::new()
    });
    let status: ProcStatus = from_document(doc).unwrap_or_else(|e| {
        error!(proc_name = desc.name, err = ?e, "status files are auto generated so should always be valid");
        Default::default()
    });
    Ok(status)
}

pub async fn update_status(desc: &ProcDesc, cmd_type: Cmd, cmd: String) -> Result<(), DdlsError> {
    let status_path = desc.dir.join("status.toml");

    let mut status = read_status(desc).await?;

    let cmd_status = match cmd_type {
        Cmd::Setup => &mut status.setup,
        Cmd::Prepare => &mut status.prepare,
        Cmd::Run => &mut status.run,
        Cmd::Update => &mut status.update,
    };
    *cmd_status = Some(CmdStatus {
        executed: cmd,
        time: Utc::now().naive_utc(),
    });
    let doc = to_document(&status).expect("should be able to serialze to toml");
    let status_str = doc.to_string();
    write(&status_path, status_str)
        .await
        .inspect_err(|e| warn!(proc_name = desc.name, err = ?e, "error updating status"))?;

    Ok(())
}
