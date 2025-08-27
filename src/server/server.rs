use std::sync::mpsc::SyncSender;

use crate::scaner::RollStuff;


pub async fn server_listen(cnf: BlkScrConfig, 
    setting: Arc<Mutex<ScanSettings>>,
    dbconn: Arc<Mutex<Connection>>,
    mut wkr: Worker,
    // diamovedate: Arc<Mutex<HashMap<DiamondName, u64>>>,
) {
    let wkr2 = wkr.clone();
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
    let app = routes(ApiCtx{cnf, setting, dbconn/*, diamovedate*/});
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = wkr.wait().await;
        })
    .await {
        println!("{e}");
    }
    println!("[Scaner] serve end.");
    wkr2.end();

}
