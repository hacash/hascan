
use basis::interface::Scaner;


mod database;
mod setting;
mod server;
mod scaner;


include!("init.rs");


fn main() -> Rerr {

    let cnfp = "./hascan.config.ini".to_string();
    let inicnf = load_config(cnfp.clone());

    // create scaner
    let cnf = scaner::BlkScrConfig::new(&inicnf);
    let (settings, dbconn) = init_db(&cnf.datadir)?;
    let mut scaner = scaner::BlkScaner::new(cnf, settings, dbconn);
    scaner.init(&inicnf)?;

    

    // start run
    hacash::run_with_scaner(&cnfp, Box::new(scaner))?;

    Ok(())
}
