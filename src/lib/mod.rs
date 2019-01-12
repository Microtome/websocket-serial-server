//! The serial support library contains all
//! the functionality to read ports, and send data
//! between threads reading serial port data
//! and threads handling websocket requests

#![recursion_limit = "1024"]
#![allow(dead_code)]
#![allow(unused_variables)]
extern crate argparse;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod cfg;
pub mod common;
pub mod dynamic_sleep;
pub mod errors;
pub mod manager;
pub mod messages;
pub mod port_manager;
pub mod sub_manager;
pub mod writelock_manager;
