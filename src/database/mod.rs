


use rusqlite::{params, Connection, Transaction as DBTransaction, Result as DBResult};


use hacash::sys::*;
use hacash::interface::field::*;
use hacash::interface::protocol::*;
use hacash::protocol::transaction::*;
use hacash::protocol::action::*;
use hacash::mint::action::*;
use hacash::core::field::*;

use crate::setting::*;

pub const COINTY_ZHU: u8 = 1;
pub const COINTY_SAT: u8 = 2;
pub const COINTY_DIA: u8 = 3;

pub const OPTY_CH_OPEN: u8 = 1; // channel open
pub const OPTY_CH_CLOSE: u8 = 2; // channel close



include!("init.rs");
include!("address.rs");
include!("transfer.rs");


