

pub async fn server_listen(cnf: BlkScrConfig, 
    setting: Arc<Mutex<ScanSettings>>,
    dbconn: Arc<Mutex<Connection>>,
    mut wkr: Worker,
    // diamovedate: Arc<Mutex<HashMap<DiamondName, u64>>>,
) {
    let port = cnf.listen;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = match TcpListener::bind(addr).await {
        Ok(v) => v,
        Err(e) => {
            println!("\n[Error] Hascan Server bind port {} error: {}\n", port, e);
            return;
        }
    };
    println!("[Hascan Server] Listening on http://{addr}");
    // 
    let app = routes(ApiCtx{cnf, setting, dbconn/*, diamovedate*/});
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = wkr.wait().await;
        })
    .await {
        println!("{e}");
    }
    println!("[Scaner] serve end.");

}
