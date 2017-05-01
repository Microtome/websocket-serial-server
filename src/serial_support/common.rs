extern crate serialport;

use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use serial_support::messages::SerialResponse;

use serialport as sp;


/// Struct for containing Port information
#[derive(Clone)]
pub struct OpenPort<'a> {
    /// The opened serial port
    port: &'a sp::SerialPort,
    /// Handle that controls writes to port
    write_handle: String,
    /// Send channel used to send responses to
    /// all clients who opened the port
    clients: &'a Vec<Sender<SerialResponse>>,
}

/// Manages tracking and reading/writing from serial
/// ports
struct SerialPortManager<'a> {
    /// Maintains list of ports
    open_ports: HashMap<String, OpenPort<'a>>,
}

impl<'a> SerialPortManager<'a> {
    fn open_port(&self, port: String) {}
    fn add_sender(&self, port: String, rcvr: Sender<SerialResponse>) {}
}

