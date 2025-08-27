

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
    fn do_start(&self, mut wkr: Worker) {
        let rlsfrx = self.rlsfrx.lock().unwrap().take().unwrap();
        loop {
            if wkr.quit() {
                println!("[Scaner] scan end.");
                return;
            }
            let Ok(stuff) = rlsfrx.recv() else {
                break;
            };
            // call toll
            let mut dbc = self.dbconn.lock().unwrap();
            let mut set = self.setting.lock().unwrap();
            // let mut dmvd = self.diamovedate.lock().unwrap();
            let block = stuff.blk.as_read();
            let csta = CoreStateRead::wrap(stuff.sta.as_ref());
            let csto = BlockStore::wrap(stuff.sto);
            let mut adrs = AddressCache::new();
            let ise = do_scan(self, &mut *set, &mut *dbc, 
                &mut adrs, block, csta, csto,
            );
            if let Err(e) = ise {
                panic!("Scaner do_scan height {} error: {}", block.height(), e);
            };
        }
        println!("[Scaner] scan end.");
        wkr.end(); // end
    }

}