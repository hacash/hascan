
#[derive(Clone)]
pub struct ApiCtx {
    pub cnf: BlkScrConfig,
    pub dbconn: Arc<Mutex<Connection>>,
    pub setting: Arc<Mutex<ScanSettings>>,
    // pub diamovedate: Arc<Mutex<HashMap<DiamondName, u64>>>,
}

impl ApiCtx {
    

}

