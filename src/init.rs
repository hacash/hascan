use std::fs;

use rusqlite::Connection;


use sys::{ self, *};
use field::*;


use crate::database::*;
use crate::setting::*;

// const DATADIR: &str = "./hacash_scan_data";

fn decode_settings(data: &[u8], source: &str) -> Ret<ScanSettings> {
    let (settings, used) = ScanSettings::decode(data)
        .map_err(|e| sys::Error::fault(format!("decode hascan {source} failed: {e}")))?;
    if used != data.len() {
        return sys::errf!(
            "decode hascan {} failed: {} trailing bytes",
            source,
            data.len() - used
        );
    }
    Ok(settings)
}

pub fn init_db(datadir: &str) -> Ret<(ScanSettings, Connection)> {

    // create data dir
    let datadir = datadir.to_owned();
    fs::create_dir_all(&datadir)
        .map_err(|e| format!("create hascan data dir {} failed: {e}", datadir))?;
    // Read the compatibility file lazily. Once SQLite has a settings snapshot,
    // a partial mirror write must not prevent startup.
    let stfn = datadir.to_owned() + "/settings.dat";
    let legacy_settings = match fs::read(&stfn) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
        Ok(data) => Ok(Some(data)),
    };
    // open sqlite db
    let mut dbconn = Connection::open(datadir.to_owned()+"/database.db3").map_err(|e|e.to_string())?;
    create_tables(&mut dbconn).map_err(|e|e.to_string())?;

    let mut settings = match load_scan_settings(&dbconn).map_err(|e| e.to_string())? {
        Some(data) => decode_settings(&data, "database settings")?,
        None => match legacy_settings {
            Ok(Some(data)) => decode_settings(&data, &format!("settings {stfn}"))?,
            Ok(None) => ScanSettings::default(),
            Err(e) => return Err(format!("read hascan settings {stfn} failed: {e}").into()),
        },
    };

    // scan height is authoritative in db (written in block tx)
    let mut db_scan_height = load_scan_height(&dbconn).map_err(|e|e.to_string())?;
    let setting_scan_height = settings.height.uint();
    if db_scan_height == 0 {
        let legacy_height = if setting_scan_height > 0 {
            setting_scan_height
        } else {
            infer_legacy_scan_height(&dbconn).map_err(|e|e.to_string())?
        };
        if legacy_height > 0 {
            eprintln!(
                "[hascan] migrating legacy checkpoint at height {}; verify or rebuild the explorer database if the old settings file was incomplete",
                legacy_height
            );
            save_scan_height_conn(&dbconn, legacy_height).map_err(|e|e.to_string())?;
            db_scan_height = legacy_height;
        }
    }
    settings.height = Uint5::from_checked(db_scan_height)
        .ok_or_else(|| sys::Error::fault("hascan database checkpoint exceeds Uint5"))?;

    // align auto_inc_address_id with persisted max primary key
    let max_acc_id = load_max_account_id(&dbconn).map_err(|e|e.to_string())?;
    if settings.auto_inc_address_id.uint() < max_acc_id {
        settings.auto_inc_address_id = Uint5::from_checked(max_acc_id)
            .ok_or_else(|| sys::Error::fault("hascan account ID exceeds Uint5"))?;
    }

    // ok ret
    Ok((settings, dbconn))

}


pub fn save_setting(datadir: &str, setting: &ScanSettings) -> Rerr {
    let stfn = datadir.to_owned() + "/settings.dat";
    fs::write(stfn, setting.encode()).map_err(|e| sys::Error::fault(e.to_string()))
}
