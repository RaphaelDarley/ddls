use std::{collections::HashMap, fs};

use axum::{routing::get, Router};
use tokio::sync::mpsc;
use toml_edit::{de::from_document, DocumentMut};
use tracing::info;

use crate::{
    manager::manager_start,
    parse::proc::{DdlsToml, ProcToml},
    proc::ProcDesc,
};

pub fn start() {
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

    let (manager_tx, manager_rx) = mpsc::channel::<()>(16);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let manager = rt.spawn(manager_start(proc_descs));
    let server = rt.block_on(serve());
    // .block_on(serve())
}

// todo impl daemonize
pub async fn serve() {
    // TODO handle signals
    let app = Router::new()
        .route("/", get(|| async {"DaeDaLuS running"}))
        // .route("/users", post(create_user))
        ;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6615").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
