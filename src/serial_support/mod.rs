//! The serial support library contains all
//! the functionality to read ports, and send data
//! between threads reading serial port data
//! and threads handling websocket requests

#![recursion_limit = "1024"]
#![allow(dead_code)]
#![allow(unused_variables)]
extern crate argparse;
extern crate base64;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serialport;
extern crate toml;
extern crate websocket;



pub mod common;
pub mod cfg;
pub mod dynamic_sleep;
pub mod errors;
pub mod manage;
pub mod messages;
pub mod port_manager;
pub mod sub_manager;
pub mod writelock_manager;
