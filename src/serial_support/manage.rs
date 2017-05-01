extern crate serialport;

use std::collections::HashMap;
use std::num::Wrapping;
use std::rc::Weak;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use serialport as sp;

use serial_support::common::*;
use serial_support::messages::*;

/// Convenience type for a listener
/// that accepts weak refs of Senders of Serial Reponses
/// This is how the manager will communicate
/// results back to the websockets
type SubscReceiver = Receiver<Weak<Sender<SerialResponse>>>;

/// Struct for containing Port information
pub struct OpenPort<'a> {
  /// The opened serial port
  port: &'a mut sp::SerialPort,
  /// Handle that controls writes to port
  write_handle: String,
  /// Send channel used to send responses to
  /// all clients who opened the port
  clients: &'a mut Vec<Sender<SerialResponse>>,
}

/// Manages tracking and reading/writing from serial
/// ports
struct SerialPortManager<'a> {
  /// Maintains list of ports
  open_ports: HashMap<String, OpenPort<'a>>,
  /// Receiver for serial requests
  receiver: Receiver<SerialRequest>,
  /// Receiver for response subscription requests
  subsc_receiver: SubscReceiver,
}

impl<'a> SerialPortManager<'a> {
  fn open_port(&self, port: String) {
    // Check if port is already open

    // If not, open it
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

  fn release_write_lock(&self, handle: String, port: Option<String>) {}

  fn add_write_lock(&self, handle: String, port: String) {}

  fn write_port(&self, handle: String, port: String, data: String, base64: Option<bool>) {}

  fn close_port(&self, port: String) {}

  /// Handles and dispatches SerialRequest sent by 
  /// the channel
  fn handle_serial_request(&self, msg: SerialRequest) {
    match msg {
      SerialRequest::Open { port } => self.open_port(port),
      SerialRequest::WriteLock { handle, port } => self.add_write_lock(handle, port),
      SerialRequest::ReleaseWriteLock { handle, port } => self.release_write_lock(handle, port),
      SerialRequest::Write {
        handle,
        port,
        data,
        base64,
      } => self.write_port(handle, port, data, base64),
      SerialRequest::Close { port } => self.close_port(port),
    }
  }

  /// Fire up the port manager
  ///
  /// TODO Right now this is spinning a thread between reading
  /// subscription requests, reading serial port data
  /// etc. This will cause high cpu usage
  ///
  /// TODO Once this is working, refactor to avoid
  /// thread spinning?
  ///
  fn run(&mut self) {

    let mut shutdown = false;
    let sleep_dur = Duration::from_millis(50);
    let mut serial_buf: Vec<u8> = vec![0; 4096];

    while !shutdown {
      // Sleep for a little bit to avoid pegging cpu
      thread::sleep(sleep_dur);

      // Handle serial operation requests
      match self.receiver.try_recv(){
        Err(e) => {
          match e {
            TryRecvError::Empty => {
              // nothing to do
            }
            TryRecvError::Disconnected => {
              // Remote end hung up, time to shutdown
              info!("Shutting down SerialPortManager");
            }
          }
        }
        Ok(req) => self.handle_serial_request(req)
      }

      // Check for new data on each port
      for (portName, openPort) in self.open_ports.iter_mut() {
        // If we have data, send it back
        let bytes_read = openPort.port.read(serial_buf.as_mut_slice());
        // Remove all dead weak references
        // TODO SEND THE DATA
        // openPort.clients.retain(|r| r.upgrade().is_some());
        // for client in openPort.clients.iter_mut(){
        //   match client.upgrade() {
        //     Some(v) => {}
        //     _ => {}
        //   }
        // }
      }

      // Check for Subscribe requests
      match self.subsc_receiver.try_recv() {
        Err(e) => {
          match e {
            TryRecvError::Empty => {
              // nothing to do
            }
            TryRecvError::Disconnected => {
              // Remote end hung up, time to shutdown
              shutdown = true;
              info!("Shutting down SerialPortManager");
            }
          }
        }
        Ok(subs) => {

        }
      }
    }
  }
}


// TODO How to spawn and fire it up
// TODO Document that closing subscription sender will result in
// portmanager shutdown.
// fn spawnManager() -> {}
