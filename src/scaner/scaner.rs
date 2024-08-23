


pub struct BlkScaner {
    pub cnf: BlkScrConfig,
    dbconn: Mutex<Connection>,
    setting: Mutex<ScanSettings>,
}

impl BlkScaner {
    pub fn new(setting: ScanSettings, dbconn: Connection) -> BlkScaner {
        BlkScaner{
            cnf: BlkScrConfig::default(),
            dbconn: dbconn.into(),
            setting: setting.into(),
        }
    }
}


impl BlockScaner for BlkScaner {

    fn init(&mut self, ini: &IniObj) -> RetErr {
        self.cnf = BlkScrConfig::new(ini)?;
        Ok(())
    } 

    // another thread
    fn start(&self) -> RetErr {
        Ok(())
    }

    fn roll(&self, blkpkg: Arc<dyn BlockPkg>,  sta: Arc<dyn State>, sto: Arc<dyn Store> ) -> RetErr {
        let mut dbc = self.dbconn.lock().unwrap();
        let mut set = self.setting.lock().unwrap();
        let block = blkpkg.objc().as_read();
        let csto = CoreStoreDisk::wrap(sto.as_ref());
        let csta = CoreStateDisk::wrap(sta.as_ref());
        let msto = MintStoreDisk::wrap(sto.as_ref());
        let msta = MintStateDisk::wrap(sta.as_ref());
        let mut adrs = AddressCache::new();
        do_scan(self, &mut *set, &mut *dbc, 
            &mut adrs,
            block, csto, csta, msto, msta,
        )
    }

}

