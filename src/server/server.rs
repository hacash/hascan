
pub async fn server_listen(cnf: BlkScrConfig, 
    setting: Arc<Mutex<ScanSettings>>,
    dbconn: Arc<Mutex<Connection>>,
) {

    let port = cnf.listen;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await;
    if let Err(ref e) = listener {
        println!("\n[Error] Hascan Server bind port {} error: {}\n", port, e);
        return
    }
    let listener = listener.unwrap();
    println!("[Hascan Server] Listening on http://{addr}");
    // 
    let app = routes(ApiCtx{cnf, setting, dbconn});
    if let Err(e) = axum::serve(listener, app).await {
        println!("{e}");
    }
}
