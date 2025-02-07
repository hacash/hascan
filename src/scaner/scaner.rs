

pub struct RollStuff {
    blk: Arc<dyn Block>, 
    sta: Arc<dyn State>, 
    sto: Arc<dyn DiskDB>
}


/////////////////////////////////////


pub struct BlkScaner {
    pub cnf: BlkScrConfig,
    dbconn: Arc<Mutex<Connection>>,
    setting: Arc<Mutex<ScanSettings>>,
    // 
    rlsftx: Mutex<Option<SyncSender<RollStuff>>>,
    // rlsfrx: Mutex<Option<Receiver<RollStuff>>>,
    // opt
    prevsavetime: Mutex<u64>,
    // diamovedate: Arc<Mutex<HashMap<DiamondName, u64>>>,
}

impl BlkScaner {
    pub fn new(setting: ScanSettings, dbconn: Connection) -> BlkScaner {
        BlkScaner{
            cnf: BlkScrConfig::default(),
            dbconn: Arc::new(Mutex::new(dbconn)),
            setting: Arc::new(Mutex::new(setting)),
            rlsftx: None.into(),
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
        let _ = crate::save_setting(&self.setting.lock().unwrap());
        let dbnn = self.dbconn.lock().unwrap();
        let _ = dbnn.cache_flush().map_err(|e|e.to_string());
    }

    // another thread
    fn start(&self) -> Rerr {
        self.do_start()
    }

    // another thread
    fn serve(&self) -> Rerr {
        self.do_serve()
    }

    fn roll(&self, blk: Arc<dyn Block>,  sta: Arc<dyn State>, sto: Arc<dyn DiskDB> ) -> Rerr {
        let stuff = RollStuff{blk, sta, sto};
        self.rlsftx.lock().unwrap().as_mut().unwrap()
            .send(stuff).map_err(|e|e.to_string())
    }



}

