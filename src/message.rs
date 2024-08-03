use tokio::sync::oneshot;

use crate::error::DdlsError;

#[derive(Debug)]
pub enum Message {
    UpdateSingleProc {
        name: String,
        ack: oneshot::Sender<Result<(), DdlsError>>,
    },
}
