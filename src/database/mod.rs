

use rusqlite::{Connection, Transaction as DBTransaction, Result as DBResult};


use field::*;
use field::interface::*;
use protocol::interface::*;
use protocol::action::*;

use crate::setting::*;

pub const COINTY_ZHU: u8 = 1;
pub const COINTY_SAT: u8 = 2;
pub const COINTY_DIA: u8 = 3;

pub const OPTY_CH_OPEN: u8 = 1; // channel open
pub const OPTY_CH_CLOSE: u8 = 2; // channel close



include!("init.rs");
include!("address.rs");
include!("transfer.rs");


