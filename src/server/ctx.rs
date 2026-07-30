
#[derive(Clone)]
pub struct ApiCtx {
    pub cnf: BlkScrConfig,
    pub dbconn: Arc<Mutex<Connection>>,
    pub setting: Arc<Mutex<ScanSettings>>,
}

impl ApiCtx {
    

}

