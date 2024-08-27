

pub struct RollStuff {
    blkpkg: Arc<dyn BlockPkg>, 
    sta: Arc<dyn State>, 
    sto: Arc<dyn Store>
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


impl BlockScaner for BlkScaner {

    fn init(&mut self, ini: &IniObj) -> RetErr {
        self.do_init(ini)
    } 

    fn exit(&self) -> RetErr {
        println!("[BlockScaner] closed to save the settings and database.");
        crate::save_setting(&self.setting.lock().unwrap())?;
        let dbnn = self.dbconn.lock().unwrap();
        dbnn.cache_flush().map_err(|e|e.to_string())
    }

    // another thread
    fn start(&self) -> RetErr {
        self.do_start()
    }

    // another thread
    fn serve(&self) -> RetErr {
        self.do_serve()
    }

    fn roll(&self, blkpkg: Arc<dyn BlockPkg>,  sta: Arc<dyn State>, sto: Arc<dyn Store> ) -> RetErr {
        let stuff = RollStuff{blkpkg, sta, sto};
        self.rlsftx.lock().unwrap().as_mut().unwrap()
            .send(stuff).map_err(|e|e.to_string())
    }



}

