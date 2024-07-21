use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DdlsError {
    #[error("Error invoking command: {cmd} : {err}")]
    CmdInvoke { cmd: String, err: String },
    #[error("Error Running command: {cmd} : {stderr}")]
    CmdRun {
        cmd: String,
        exit: Option<i32>,
        stderr: String,
    },
    #[error("Error doing IO: {0:?}")]
    IoError(io::Error),
}

impl From<io::Error> for DdlsError {
    fn from(value: io::Error) -> Self {
        DdlsError::IoError(value)
    }
}
