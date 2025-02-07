
/*

step1: 
sudo apt-get install libsqlite3-dev 


RUSTFLAGS="$RUSTFLAGS -Awarnings" cargo build && cp ./target/debug/hascan ./ && ./hascan
rm -rf hacash_*_data/
RUSTFLAGS="$RUSTFLAGS -Awarnings" cargo build --release && cp ./target/release/hascan ./ && ./hascan


*/

use chain::interface::Scaner;


mod database;
mod setting;
mod server;
mod scaner;


include!("init.rs");


fn main() -> Rerr {

    let cnfp = "./hascan.config.ini".to_string();
    let inicnf = load_config(cnfp);

    // scaner
    let cnf = scaner::BlkScrConfig::new(&inicnf);
    let (settings, dbconn) = init_db(&cnf.datadir)?;
    let mut scaner = scaner::BlkScaner::new(cnf, settings, dbconn);
    scaner.init(&inicnf).unwrap();

    // start run
    hacash::fullnode_with_scaner(inicnf, Box::new(scaner));

    Ok(())
}