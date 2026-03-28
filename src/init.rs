use std::fs;

use rusqlite::Connection;


use sys::{ self, *};
use field::*;


use crate::database::*;
use crate::setting::*;

// const DATADIR: &str = "./hacash_scan_data";

pub fn init_db(datadir: &str) -> Ret<(ScanSettings, Connection)> {

    // create data dir
    let datadir = datadir.to_owned();
    let _ = fs::create_dir(&datadir);
    // settings
    let stfn = datadir.to_owned() + "/settings.dat";
    let ldf = fs::read(&stfn);
    let mut settings = match ldf {
        Err(..) => ScanSettings::default(),
        Ok(dat) => ScanSettings::must(&dat),
    };
    // open sqlite db
    let mut dbconn = Connection::open(datadir.to_owned()+"/database.db3").map_err(|e|e.to_string())?;
    create_tables(&mut dbconn).map_err(|e|e.to_string())?;

    // scan height is authoritative in db (written in block tx)
    let mut db_scan_height = load_scan_height(&dbconn).map_err(|e|e.to_string())?;
    let setting_scan_height = settings.height.uint();
    if db_scan_height == 0 && setting_scan_height > 0 {
        // bootstrap from existing settings on first migration
        save_scan_height_conn(&dbconn, setting_scan_height).map_err(|e|e.to_string())?;
        db_scan_height = setting_scan_height;
    }
    settings.height = Uint5::from(db_scan_height);

    // align auto_inc_address_id with persisted max primary key
    let max_acc_id = load_max_account_id(&dbconn).map_err(|e|e.to_string())?;
    if settings.auto_inc_address_id.uint() < max_acc_id {
        settings.auto_inc_address_id = Uint5::from(max_acc_id);
    }

    // ok ret
    Ok((settings, dbconn))

}


pub fn save_setting(datadir: &str, setting: &ScanSettings) -> Rerr {
    let stfn = datadir.to_owned() + "/settings.dat";
    fs::write(stfn, setting.serialize()).map_err(|e|e.to_string())
}
