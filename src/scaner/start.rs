

impl BlkScaner {

    fn do_init(&mut self, ini: &IniObj) -> RetErr {
        self.cnf = BlkScrConfig::new(ini)?;
        // set synchronous
        let synchronous = self.cnf.synchronous.clone();
        if synchronous != "NORMAL" {
            let sql = format!("PRAGMA synchronous = {};", &synchronous);
            self.dbconn.lock().unwrap().execute(&sql, ()).map_err(|e|e.to_string())?;
        }

        Ok(())
    } 

    // another thread
    fn do_start(&self) -> RetErr {
        // roll thread
        let (sender, receiver) = sync_channel(50);
        {
            let mut rlsftx = self.rlsftx.lock().unwrap();
            *rlsftx = Some(sender);
        }
        loop {
            let stuff = receiver.recv().unwrap();
            // call toll
            let mut dbc = self.dbconn.lock().unwrap();
            let mut set = self.setting.lock().unwrap();
            let block = stuff.blkpkg.objc().as_read();
            let csto = CoreStoreDisk::wrap(stuff.sto.as_ref());
            let csta = CoreStateDisk::wrap(stuff.sta.as_ref());
            let msto = MintStoreDisk::wrap(stuff.sto.as_ref());
            let msta = MintStateDisk::wrap(stuff.sta.as_ref());
            let mut adrs = AddressCache::new();
            do_scan(self, &mut *set, &mut *dbc, 
                &mut adrs,
                block, csto, csta, msto, msta,
            )?;
        }
        errf!("cannot end of start loop")
    }

}