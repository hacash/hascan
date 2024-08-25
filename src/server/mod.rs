use std::sync::{ Arc, Mutex };
use std::net::SocketAddr;



use tokio::net::TcpListener;
use rusqlite::{ Connection };
use serde_json::json;
use axum::{
    routing::get,
    extract::{Query, Request, State},
    Router,
    response::{ IntoResponse },

};


use hacash::server::ctx::*;


use crate::scaner::BlkScrConfig;
use crate::setting::ScanSettings;



include!("ctx.rs");
include!("route.rs");
include!("active.rs");
include!("ranking.rs");
include!("server.rs");

