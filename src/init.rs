use std::fs;

use rusqlite::{params, Connection, Result};


use hacash::interface::field::*;

use hacash::sys::{ self, *};
use hacash::base::field::*;
use hacash::base::combo::*;

use crate::database::*;
use crate::setting::*;



fn init_db() -> Ret<(ScanSettings, Connection)> {

    // create data dir
    let datadir = "./hacash_scan_data".to_owned();
    fs::create_dir(&datadir);
    // settings
    let stfn = datadir.clone() + "/settings.dat";
    let ldf = fs::read(&stfn);
    let settings = match ldf {
        Err(..) => ScanSettings::default(),
        Ok(dat) => ScanSettings::must(&dat),
    };
    fs::write(stfn, settings.serialize());
    // open sqlite db
    let mut dbconn = Connection::open(datadir+"/database.db3").map_err(|e|e.to_string())?;
    create_tables(&mut dbconn).map_err(|e|e.to_string())?;
    // ok ret
    Ok((settings, dbconn))

}

