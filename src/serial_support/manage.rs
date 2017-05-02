

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use serialport as sp;
use base64;

use serial_support::messages::*;

/// Convenience type for a listener
/// that accepts weak refs of Senders of Serial Reponses
/// This is how the manager will communicate
/// results back to the websockets
type SubscReceiver = Receiver<SubscriptionRequest>;

/// Struct for containing Port information
pub struct OpenPort {
  /// Handle that controls writes to port
  write_lock_sub_id: String,
  /// The opened serial port
  /// SerialPort is not Sized, so it makes hashmap mad
  /// and so we deal with these shennanigans
  port: Rc<RefCell<sp::SerialPort>>,
}

pub struct Subscription {
  /// Subscription
  subscriber: Sender<SerialResponse>,
  /// The ports it is subscribed to
  ports: Vec<String>,
  /// Subscription id
  sub_id: String,
}

/// Manages tracking and reading/writing from serial
/// ports
struct SerialPortManager {
  /// List of port names to subscribers
  subscriptions: Vec<Subscription>,
  /// Receiver for serial requests
  receiver: Receiver<SerialRequest>,
  /// Receiver for response subscription requests
  subsc_receiver: SubscReceiver,
  /// Maintains list of ports
  open_ports: HashMap<String, OpenPort>,
}

impl SerialPortManager {
  fn open_port(&self, sub_id: String, port: String) {
    // Check if port is already open

    // If not, open it
    let sp_settings = sp::SerialPortSettings {
      baud_rate: sp::BaudRate::Baud115200,
      data_bits: sp::DataBits::Eight,
      flow_control: sp::FlowControl::None,
      parity: sp::Parity::None,
      stop_bits: sp::StopBits::One,
      timeout: Duration::from_millis(1),
    };



  }

  fn add_sender(&self, port: String, sender: Sender<SerialResponse>) {

  }

  fn create_write_lock(&self, sub_id: String, port: String) {

  }

  fn release_write_lock(&self, sub_id: String, port: Option<String>) {

  }

  fn write_port(&self, sub_id: String, port: String, data: String, base64: bool) {

  }

  fn close_port(&self, sub_id: String, port: Option<String>) {}

  /// Handles and dispatches SerialRequest sent by
  /// the channel
  fn handle_serial_request(&self, msg: SerialRequest) {
    match msg {
      SerialRequest::Open { sub_id, port } => self.open_port(sub_id, port),
      SerialRequest::WriteLock { sub_id, port } => self.add_write_lock(sub_id, port),
      SerialRequest::ReleaseWriteLock { sub_id, port } => self.release_write_lock(sub_id, port),
      SerialRequest::Write {
        sub_id,
        port,
        data,
        base64,
      } => self.write_port(sub_id, port, data, base64.unwrap_or(false)),
      SerialRequest::Close { sub_id, port } => self.close_port(sub_id, port),
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
    // Check about 30 times a second
    let sleep_dur = Duration::from_millis(33);
    let mut serial_buf: Vec<u8> = vec![0; 4096];

    while !shutdown {
      // Sleep for a little bit to avoid pegging cpu
      thread::sleep(sleep_dur);

      // Handle serial operation requests
      match self.receiver.try_recv() {
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
        Ok(req) => self.handle_serial_request(req),
      }

      // Check for new data on each port
      for (port_name, open_port) in self.open_ports.iter() {
        // If we have data, send it back
        // Read needs to borrow &mut self, so wrap port in RC?
        let result = match open_port
                .port
                .borrow_mut()
                .read(serial_buf.as_mut_slice()) {
          Ok(bytes_read) => {
            // We got some data
            let bytes = &serial_buf[0..bytes_read];
            // Try and parse the bytes as utf-8
            match String::from_utf8(bytes.to_vec()) {
              // We need to send as binary
              Err(_) => {
                SerialResponse::Read {
                  port: port_name.to_string(),
                  data: base64::encode(bytes),
                  base64: Some(true),
                }
              }
              Ok(s) => {
                SerialResponse::Read {
                  port: port_name.to_string(),
                  data: s,
                  base64: Some(false),
                }
              }
            }
          }
          Err(_) => {
            // We failed to read the port, send error
            SerialResponse::Error {
              kind: ErrorType::ReadError,
              msg: "Error reading serial port '{}'".to_string(),
              detail: None,
              port: Some(port_name.to_string()),
              sub_id: None,
            }
          }
        };

        // Send report, retain only live connections
        self
          .subscriptions
          .retain(|c| {
            // If its not the port we are currently interested in, we keep it
            if !c.ports.contains(&port_name) {
              true
            } else {
              // Otherwise try and send on it, and if it fails
              // remove the subscription
              c.subscriber
                .send(result.clone())
                .map(|_| true)
                .map_err(|_| {
                           warn!("Connection closed.");
                           false
                         })
                .unwrap()
            }
          });
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
        Ok(subReq) => {
          self
            .subscriptions
            .push(Subscription {
                    ports: Vec::with_capacity(4),
                    subscriber: subReq.subscriber,
                    sub_id: subReq.sub_id,
                  });
        }
      }
    }
  }
}



// TODO How to spawn and fire it up
// TODO Document that closing subscription sender will result in
// portmanager shutdown.
// fn spawnManager() -> {}
