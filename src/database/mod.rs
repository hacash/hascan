use rusqlite::{Connection, Result as DBResult, Transaction as DBTransaction};

use base::{Action, Transaction};
use field::*;
use mint::action_channel::{ChannelClose, ChannelOpen};
use mint::action_diamond::DiamondMint;
use mint::action_asset::AssetCreate;
use protocol::action_std::*;
use vm::action::{ContractDeploy, ContractUpdate};
use vm::ContractAddress;

use crate::setting::*;

fn db_fault(message: impl Into<String>) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(message.into())))
}

fn db_overflow(name: &str) -> rusqlite::Error {
    db_fault(format!("hascan {name} overflow"))
}

pub const COINTY_ZHU: u8 = 1;
pub const COINTY_SAT: u8 = 2;
pub const COINTY_DIA: u8 = 3;

pub const OPTY_CH_OPEN: u8 = 1; // channel open
pub const OPTY_CH_CLOSE: u8 = 2; // channel close

include!("init.rs");
include!("address.rs");
include!("ecosystem.rs");
include!("transfer.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_legacy_checkpoint_from_index_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        create_tables(&mut conn).unwrap();
        conn.execute(
            "INSERT INTO coin_transfer (height,from_aid,to_aid,coin_type,coin_amt) \
             VALUES (123,1,2,1,10000)",
            (),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO defi_operate (height,kind,aid1,aid2,tarid,data) \
             VALUES (456,1,1,2,X'00','')",
            (),
        )
        .unwrap();
        assert_eq!(infer_legacy_scan_height(&conn).unwrap(), 456);
    }
}
