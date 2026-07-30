pub mod database;
pub mod scaner;
pub mod server;
pub mod setting;

include!("init.rs");

pub fn open(ini: &sys::IniObj) -> sys::Ret<scaner::BlkScaner> {
    let cnf = scaner::BlkScrConfig::new(ini)?;
    let (settings, dbconn) = init_db(&cnf.datadir)?;
    scaner::BlkScaner::new(cnf, settings, dbconn)
}

#[cfg(test)]
mod tests {
    use field::Encode;

    use super::*;

    #[test]
    fn settings_decoder_rejects_trailing_data() {
        let mut data = setting::ScanSettings::default().encode();
        data.push(0);
        assert!(decode_settings(&data, "test settings").is_err());
    }
}
