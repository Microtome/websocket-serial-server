extern crate serialport;

use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serialport as sp;

use serial_support::messages::SerialResponse;


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
  fn open_port(&self, port: String) {
    let spSettings = sp::SerialPortSettings {
      baud_rate: sp::BaudRate::Baud115200,
      data_bits: sp::DataBits::Eight,
      flow_control: sp::FlowControl::None,
      parity: sp::Parity::None,
      stop_bits: sp::StopBits::One,
      timeout: Duration::from_millis(1),
    };


  }
  fn add_sender(&self, port: String, sender: Sender<SerialResponse>) {}
  fn remove_sender(&self, sender: Sender<SerialResponse>) {}
  fn remove_write_lock(&self, handle: String, port: Option<String>) {}
  fn add_write_lock(&self, handle: String, port: String) {}
}
