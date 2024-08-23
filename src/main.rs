
/*

step1: 
sudo apt-get install libsqlite3-dev 


RUSTFLAGS="$RUSTFLAGS -Awarnings" cargo build && cp ./target/debug/hascan ./ && ./hascan


*/

#[macro_use]
extern crate hacash;

mod database;
mod setting;
mod scaner;


include!("init.rs");


fn main() -> RetErr {

    let (settings, dbconn) = init_db()?;

    // scaner
    let scaner = scaner::BlkScaner::new(settings, dbconn);

    // start run
    hacash::run::fullnode(Some(Box::new(scaner)));

    Ok(())
}