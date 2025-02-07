use std::fs;

use rusqlite::Connection;


use sys::{ self, *};
use field::interface::*;


use crate::database::*;
use crate::setting::*;

const DATADIR: &str = "./hacash_scan_data";

pub fn init_db() -> Ret<(ScanSettings, Connection)> {

    // create data dir
    let datadir = DATADIR.to_owned();
    fs::create_dir(&datadir).unwrap();
    // settings
    let stfn = datadir.to_owned() + "/settings.dat";
    let ldf = fs::read(&stfn);
    let settings = match ldf {
        Err(..) => ScanSettings::default(),
        Ok(dat) => ScanSettings::must(&dat),
    };
    // open sqlite db
    let mut dbconn = Connection::open(datadir.to_owned()+"/database.db3").map_err(|e|e.to_string())?;
    create_tables(&mut dbconn).map_err(|e|e.to_string())?;
    // ok ret
    Ok((settings, dbconn))

}


pub fn save_setting(setting: &ScanSettings) -> Rerr {
    let stfn = DATADIR.to_owned() + "/settings.dat";
    fs::write(stfn, setting.serialize()).map_err(|e|e.to_string())
}

