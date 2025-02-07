

impl BlkScaner {

    fn do_init(&mut self, _ini: &IniObj) -> Rerr {
        // set synchronous
        let synchronous = self.cnf.synchronous.clone();
        if synchronous != "NORMAL" {
            let sql = format!("PRAGMA synchronous = {};", &synchronous);
            self.dbconn.lock().unwrap().execute(&sql, ()).map_err(|e|e.to_string())?;
        }
        Ok(())
    } 

    // another thread
    fn do_start(&self) -> Rerr {
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
            // let mut dmvd = self.diamovedate.lock().unwrap();
            let block = stuff.blk.as_read();
            let csta = CoreStateRead::wrap(stuff.sta.as_ref());
            let csto = BlockDisk::wrap(stuff.sto);
            let mut adrs = AddressCache::new();
            do_scan(self, &mut *set, &mut *dbc, 
                &mut adrs, block, csta, csto,
            )?;
        }
    }

}