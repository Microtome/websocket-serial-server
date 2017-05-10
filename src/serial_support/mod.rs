//! The serial support library contains all
//! the functionality to read ports, and send data
//! between threads reading serial port data 
//! and threads handling websocket requests 

// TODO Remove once dev is done
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod messages;
pub mod errors;
mod port_manager;
mod writelock_manager;
mod sub_manager;
mod common;
pub mod manage;
