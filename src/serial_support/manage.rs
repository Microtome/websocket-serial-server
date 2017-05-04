

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use serialport as sp;
use base64;

use serial_support::errors::*;
use serial_support::messages::*;

/// Convenience type for a listener
/// that accepts weak refs of Senders of Serial Reponses
/// This is how the manager will communicate
/// results back to the websockets
type SubscReceiver = Receiver<SubscriptionRequest>;

/// Struct for containing Port information
pub struct OpenPort {
  /// Handle that controls writes to port
  write_lock_sub_id: Option<String>,
  /// The opened serial port
  /// SerialPort is not Sized, so it makes hashmap mad
  /// and so we deal with these shennanigans
  port: RefCell<Box<sp::SerialPort>>,
}

/// Subscription
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
///
/// TODO efficiency can be improved through choice of better
/// data structures though looping through a short list
/// is probably faster than hashing...
struct SerialPortManager {
  /// Receiver for serial requests
  receiver: Receiver<SerialRequest>,
  /// Receiver for response subscription requests
  subsc_receiver: SubscReceiver,
  /// List of port names to subscribers
  subscriptions: Vec<Subscription>,
  /// Maintains list of ports
  open_ports: HashMap<String, OpenPort>,
}

impl SerialPortManager {
  // /// Try and get the subscription for sub_id
  // /// If unsuccessful, return a Err<ErrorKind::SubscriptionNotFound>
  // fn get_subscription(&mut self, sub_id: & String) -> Result<&mut Subscription> {
  //   match self.subscriptions.iter_mut().find(|s| & s.sub_id == sub_id) {
  //     None => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into()),
  //     Some(sub) => Ok(sub)
  //   }
  // }

  // /// Try and get the serial port
  // /// If unsuccessful, return a Err<ErrorKind::OpenPortNotFound>
  // fn get_port(&mut self, port: &String) -> Result<&mut OpenPort> {
  //   match self.open_ports.get_mut(port) {
  //     None => Err(ErrorKind::OpenPortNotFound(port.to_string()).into()),
  //     Some(sp) => Ok(sp)
  //   }
  // }

  /// Open the given port.
  /// If the port is already open, then just subscribe to it
  fn open_port(&mut self, sub_id: & String, port_name:& String) -> Result<SerialResponse>  {

    fn open_serial_port(port:& String) -> Result<Box<sp::SerialPort>>{
      // If not, open it
      let sp_settings = sp::SerialPortSettings {
        baud_rate: sp::BaudRate::Baud115200,
        data_bits: sp::DataBits::Eight,
        flow_control: sp::FlowControl::None,
        parity: sp::Parity::None,
        stop_bits: sp::StopBits::One,
        timeout: Duration::from_millis(1),
      };
      sp::open_with_settings(&port, &sp_settings).map_err(|err|ErrorKind::SerialportError(err).into())
    };

    let sub = self.
      subscriptions.
      iter_mut().
      find(|s| & s.sub_id == sub_id).
      ok_or(ErrorKind::SubscriptionNotFound(sub_id.to_string()))?;
    
    let mut sp = try!(open_serial_port(port_name));

    let port = self.open_ports.get_mut(port_name).or_else(||{
        let op = OpenPort{write_lock_sub_id: None, port:RefCell::new(sp)};
        self.open_ports.insert(port_name.to_string(), op);
        Some(& mut op)
      });
    
    match sub.ports.iter().position(|p| p == port_name) {
      None => {sub.ports.push(port_name.to_string())}
      _ => {debug!("Port '{}' already subscribed by sub '{}' ", port_name, sub_id)}
    };
    
    Ok(SerialResponse::Opened{
      port: port_name.to_string()
    })

    // if subscription.ports.contains(port){
    //   return 
    // }

    // self.get_port(port)?.map_error()


    // self.get_subscription(sub_id)?.

    // match self.subscriptions.iter().position(|s| s.sub_id == sub_id){
    //   // We don't even know who to send an error to...
    //   None => warn!("No subscription found for sub_id {}",sub_id),
    //   Some(idx) => {
    //     let sub = &mut self.subscriptions[idx];
    //     let serial_port = match self.open_ports.get(&port){
    //       None => {
    //         let sp = sp::open_with_settings(&port, &sp_settings);
    //         self.open_ports.insert(port, sp);
    //         return
    //       }
    //       Some(sp) => Ok(sp.port.borrow_mut())
    //     };

    //   }
    // }

    // match self.open_ports.get(&port) {
    //   case Some(serial_port) => {

    //   },
    //   case None => {

    //   }
    // }

  }

  // fn add_sender(&self, port: String, sender: Sender<SerialResponse>) {}

  fn set_write_lock(&self, sub_id: String, port: String) {}

  fn release_write_lock(&self, sub_id: String, port: Option<String>) {}

  fn write_port(&mut self, sub_id: String, port: String, data: String, base64: bool) {
    // get port

    // check sub_id matches write_lock id

    // If it does, try and write

    // Else send a write lock error
  }

  /// Send a response to a subscription with the id sub_id
  fn send_response(&mut self, sub_id: String, response: SerialResponse) {
    match self
            .subscriptions
            .iter()
            .position(|s| s.sub_id == sub_id) {
      Some(idx) => {
        match self.subscriptions[idx].subscriber.send(response.clone()) {
          Ok(_) => debug!("Sucessfully sent {:?} to {}", response.clone(), sub_id),
          Err(_) => {
            debug!("Remote end for sub_id {} closed, removing", sub_id);
            self.subscriptions.remove(idx);
          }
        }
      }
      None => warn!("No subscription found for sub_id '{}'", sub_id),
    }
  }

  /// Broadcast a response to all subscriptions
  fn broadcast(&mut self, response: SerialResponse) {
    self
      .subscriptions
      .retain(|sub| {
        sub
          .subscriber
          .send(response.clone())
          .map(|_| true)
          .unwrap_or_else(|_| {
                            warn!("Connection closed.");
                            false
                          })
      });
  }

  fn close_port(&self, sub_id: String, port: Option<String>) {}

  /// Handles and dispatches SerialRequest sent by
  /// the channel
  fn handle_serial_request(&mut self, msg: SerialRequest) {
    // match msg {
    //   SerialRequest::Open { sub_id, port } => self.open_port(sub_id, port),
    //   SerialRequest::WriteLock { sub_id, port } => self.set_write_lock(sub_id, port),
    //   SerialRequest::ReleaseWriteLock { sub_id, port } => self.release_write_lock(sub_id, port),
    //   SerialRequest::Write {
    //     sub_id,
    //     port,
    //     data,
    //     base64,
    //   } => self.write_port(sub_id, port, data, base64.unwrap_or(false)),
    //   SerialRequest::Close { sub_id, port } => self.close_port(sub_id, port),
    // }
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
              Err(e) => Err(ErrorKind::Utf8Error(e)),
              Ok(s) => {
                Ok(SerialResponse::Read {
                     port: port_name.to_string(),
                     data: s,
                     base64: Some(false),
                   })
              }
            }
          }
          Err(_) => Err(ErrorKind::PortReadError(port_name.to_string()).into()),
        };

        let r = result.unwrap_or_else(|ek| to_serial_response_error(ek.into()));          

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
                .send(r.clone())
                .map(|_| true)
                .unwrap_or_else(|_| {
                                  warn!("Connection was closed.");
                                  false
                                })
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
        Ok(sub_req) => {
          self
            .subscriptions
            .push(Subscription {
                    ports: Vec::with_capacity(4),
                    subscriber: sub_req.subscriber,
                    sub_id: sub_req.sub_id,
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
