

pub struct RollStuff {
    blk: Arc<dyn Block>, 
    sta: Arc<Box<dyn State>>, 
    sto: Arc<dyn DiskDB>
}


/////////////////////////////////////


pub struct BlkScaner {
    pub cnf: BlkScrConfig,
    dbconn: Arc<Mutex<Connection>>,
    setting: Arc<Mutex<ScanSettings>>,
    // 
    rlsftx: Mutex<Option<SyncSender<RollStuff>>>,
    rlsfrx: Mutex<Option<Receiver<RollStuff>>>,
    // opt
    prevsavetime: Mutex<u64>,
    // diamovedate: Arc<Mutex<HashMap<DiamondName, u64>>>,
}

impl BlkScaner {
    pub fn new(cnf: BlkScrConfig, setting: ScanSettings, dbconn: Connection) -> BlkScaner {
        // roll thread
        let (sender, receiver) = sync_channel(50);
        BlkScaner{
            cnf,
            dbconn: Arc::new(Mutex::new(dbconn)),
            setting: Arc::new(Mutex::new(setting)),
            rlsftx: Some(sender).into(),
            rlsfrx: Some(receiver).into(),
            prevsavetime: 0.into(),
            // diamovedate: Arc::default(),
        }
    }
}


impl Scaner for BlkScaner {

    fn init(&mut self, ini: &IniObj) -> Rerr {
        self.do_init(ini)
    } 

    fn exit(&self) {
        println!("[BlockScaner] closed to save the settings and database.");
        let _ = crate::save_setting(&self.cnf.datadir, &self.setting.lock().unwrap());
        let dbnn = self.dbconn.lock().unwrap();
        let _ = dbnn.cache_flush().map_err(|e|e.to_string());
    }

    // another thread
    fn start(&self, wkr: Worker) {
        let rt = node::new_tokio_rt( false );
        let _ = rt.block_on(async move {
            self.do_start(wkr)
        });
    }

    // another thread
    fn serve(&self, wkr: Worker) {
        self.do_serve(wkr)
    }

    fn roll(&self, blk: Arc<dyn Block>, sta: Arc<Box<dyn State>>, sto: Arc<dyn DiskDB> ) {
        let stuff = RollStuff{blk, sta, sto};
        let sdres = self.rlsftx.lock().unwrap().as_mut().unwrap().send(stuff);
        if let Err(e) = sdres {
            panic!("~ ~ ~ ~ Block Scaner do roll send block stuff error: {}", e)
            // println!("~ ~ ~ ~ Block Scaner do roll send block stuff error: {}", e)
        }
    }



}

