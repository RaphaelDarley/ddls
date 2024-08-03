pub async fn update(endpoint: String, port: u16, target: String) {
    // send to http://<url>/update/<target>
    reqwest::get(format!("http://{endpoint}:{port}/update/{target}"))
        .await
        .unwrap();
}
