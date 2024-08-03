use std::{collections::HashMap, fs};

use axum::{body::Body, extract::Path, routing::get, Extension, Router};
use tokio::sync::{mpsc, oneshot};
use toml_edit::{de::from_document, DocumentMut};
use tracing::info;

use crate::{
    manager::manager_start,
    message::Message,
    parse::proc::{DdlsToml, ProcToml},
    proc::ProcDesc,
};

pub async fn start() {
    info!("called start");
    let toml_text = fs::read_to_string("ddls.toml").expect("ddls.toml file should be present");
    let toml = toml_text
        .parse::<DocumentMut>()
        .expect("config should be valid toml");
    let ddls_toml: DdlsToml = from_document(toml).expect("config file should be properly formed");
    for p in &ddls_toml.proc {
        // println!("proc {} in toml", p.0);
        info!(parsed_proc = p.0);
    }
    let proc_descs = ddls_toml.proc.into_iter().map(|(n, t)| t.to_desc(n));

    let (manager_tx, manager_rx) = mpsc::channel::<Message>(16);

    // let rt = tokio::runtime::Builder::new_multi_thread()
    //     .enable_all()
    //     .build()
    //     .unwrap();
    let _manager = tokio::spawn(manager_start(proc_descs, manager_rx));
    let _server = serve(manager_tx).await;
    // .block_on(serve())
}

// todo impl daemonize
pub async fn serve(manager_tx: mpsc::Sender<Message>) {
    // TODO handle signals
    let app = Router::new()
        // .route("/", get(root_get_handler))
        .route("/update/:target", get(update_get_handler))
        // .route("/users", post(create_user))
        .layer(Extension(manager_tx));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6615").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn update_get_handler(
    mtx: Extension<mpsc::Sender<Message>>,
    Path(target): Path<String>,
    _body: Body,
) {
    let (ack_tx, ack_rx) = oneshot::channel();
    mtx.send(Message::UpdateSingleProc {
        name: target,
        ack: ack_tx,
    })
    .await
    .expect("sending to manager should succeed");

    let ack = ack_rx.await;
    info!(?ack, "got ack");
}
