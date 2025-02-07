
/*

step1: 
sudo apt-get install libsqlite3-dev 


RUSTFLAGS="$RUSTFLAGS -Awarnings" cargo build && cp ./target/debug/hascan ./ && ./hascan
rm -rf hacash_*_data/
RUSTFLAGS="$RUSTFLAGS -Awarnings" cargo build --release && cp ./target/release/hascan ./ && ./hascan


*/


mod database;
mod setting;
mod server;
mod scaner;


include!("init.rs");


fn main() -> Rerr {

    let cnfp = "./hascan.config.ini".to_string();
    let inicnf = load_config(cnfp);

    // scaner
    let (settings, dbconn) = init_db()?;
    let scaner = scaner::BlkScaner::new(settings, dbconn);

    // start run
    hacash::fullnode_with_scaner(inicnf, Box::new(scaner));

    Ok(())
}