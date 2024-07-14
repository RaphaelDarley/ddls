use axum::{routing::get, Router};

pub fn start() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(serve())
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
