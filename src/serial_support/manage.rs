/// Serial port management module supporting one
/// writer and multiple readers
///
/// Clients can lock a port for writing, but
/// subscribe to data from multiple ports for reads

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use serialport as sp;
use base64;

use serial_support::errors::*;
use serial_support::messages::*;
use serial_support::sub_manager::*;
use serial_support::port_manager::*;
use serial_support::writelock_manager::*;
use serial_support::common::*;

/// Manager manages connection state.
struct Manager{
  /// Manage write lock status
  writelock_manager: WriteLockManager,
  /// Manage ports
  port_manager: PortManager,
  /// Manage subscriptions
  sub_manager: SubscriptionManager,
  /// Receiver for serial requests
  receiver: Receiver<SerialRequest>,
  /// Receiver for response subscription requests
  subsc_receiver: SubscReceiver
}

impl Manager{

  // Constructor
  pub fn new(receiver: Receiver<SerialRequest>,  subsc_receiver: SubscReceiver) -> Manager{
    Manager{
      writelock_manager: WriteLockManager::new(),
      port_manager: PortManager::new(),
      sub_manager: SubscriptionManager::new(),
      receiver: receiver,
      subsc_receiver: subsc_receiver
    }
  }

  /// Spawn an instance in a new thread.
  pub fn spawn(receiver: Receiver<SerialRequest>,  subsc_receiver: SubscReceiver) -> thread::JoinHandle<()>{
    thread::spawn( move ||{
      Manager::new(receiver,subsc_receiver).run();
    })
  }

  /// Close out ports we've had bad read / write issues with
  pub fn close_bad_ports(ports: Vec<String>){
    // Try and close the ports
    // Notify subscribers
    unimplemented!();
  }

  /// Handles and dispatches SerialRequest sent by
  /// the channel
  fn handle_serial_request(&self, msg: SerialRequest) {
    unimplemented!();
    // match msg {
    //   SerialRequest::Open { sub_id, port } => self.port_manager.open_port(sub_id, port),
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


  /// Main loop
  fn run(&self){

    // Shutdown flag
    let mut shutdown = false;
    // Check about 30 times a second
    let sleep_dur = Duration::from_millis(33);

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

      // Bad ports we couldn't read from
      let mut badPorts = Vec::<String>::new();

      for (port_name, result) in self.port_manager.read_all_ports(){
        match result {
          Ok(data) => {
            let response = match String::from_utf8(data) {
              // We need to send as binary
              Err(e) => SerialResponse::Read {
                
                     port: port_name.to_string(),
                     data: base64::encode(&e.into_bytes()),
                     base64: Some(true),
                   },
              Ok(s) => SerialResponse::Read {
                     port: port_name.to_string(),
                     data: s,
                     base64: Some(false),
                   }
              };
            },
          Err(_) => badPorts.push(port_name)
        }
      }

      // We're not finished yet
      unimplemented!();

      // Close bad ports, notify clients


      // for (port_name, open_port) in self.open_ports.iter() {
      //   // If we have data, send it back
      //   // Read needs to borrow &mut self, so wrap port in RC?
      //   let result = match open_port
      //           .port
      //           .borrow_mut()
      //           .read(serial_buf.as_mut_slice()) {
      //     Ok(bytes_read) => {
      //       // We got some data
      //       let bytes = &serial_buf[0..bytes_read];
      //       // Try and parse the bytes as utf-8
      //       match String::from_utf8(bytes.to_vec()) {
      //         // We need to send as binary
      //         Err(e) => Err(ErrorKind::Utf8Error(e)),
      //         Ok(s) => {
      //           Ok(SerialResponse::Read {
      //                port: port_name.to_string(),
      //                data: s,
      //                base64: Some(false),
      //              })
      //         }
      //       }
      //     }
      //     Err(_) => Err(ErrorKind::PortReadError(port_name.to_string()).into()),
      //   };

    //     let r = result.unwrap_or_else(|ek| to_serial_response_error(ek.into()));          

    //     // Send report, retain only live connections
    //     self
    //       .subscriptions
    //       .retain(|c| {
    //         // If its not the port we are currently interested in, we keep it
    //         if !c.ports.contains(&port_name) {
    //           true
    //         } else {    
    //           // Otherwise try and send on it, and if it fails
    //           // remove the subscription
    //           c.subscriber
    //             .send(r.clone())
    //             .map(|_| true)
    //             .unwrap_or_else(|_| {
    //                               warn!("Connection was closed.");
    //                               false
    //                             })
    //         }
    //       });
    //   }

    //   // Check for Subscribe requests
    //   match self.subsc_receiver.try_recv() {
    //     Err(e) => {
    //       match e {
    //         TryRecvError::Empty => {
    //           // nothing to do
    //         }
    //         TryRecvError::Disconnected => {
    //           // Remote end hung up, time to shutdown
    //           shutdown = true;
    //           info!("Shutting down SerialPortManager");
    //         }
    //       }
    //     }
    //     Ok(sub_req) => {
    //       self
    //         .subscriptions
    //         .push(Subscription {
    //                 ports: Vec::with_capacity(4),
    //                 subscriber: sub_req.subscriber,
    //                 sub_id: sub_req.sub_id,
    //               });
    //     }
    //   }
    // }
    // }
    }
  }  
}




// /// Subscriptions keep track of which client is interested
// /// in which port
// impl Subscription {

//   /// Is this subscription already subscribed to the port
//   fn has_port(&self, port_name: &String) -> bool{
//     self.ports.iter().find(|pn| *pn == port_name).is_some()
//   }

//   /// If this subscription is not already subscribed to 
//   /// port_name, then add it
//   fn add_if_not_present(&mut self, port_name: &String){
//     if !self.has_port(&port_name){
//       self.ports.push(*port_name)
//     }
//   }
// }

// /// Manages tracking and reading/writing from serial
// /// ports
// ///
// /// TODO efficiency can be improved through choice of better
// /// data structures though looping through a short list
// /// is probably faster than hashing...
// struct SerialPortManager {
//   /// Receiver for serial requests
//   receiver: Receiver<SerialRequest>,
//   /// Receiver for response subscription requests
//   subsc_receiver: SubscReceiver,
//   /// List of port names to subscribers
//   subscriptions: Vec<Subscription>,
//   /// Maintains list of ports
//   open_ports: HashMap<String, OpenPort>,
// }

// impl SerialPortManager {
//   // /// Try and get the subscription for sub_id
//   // /// If unsuccessful, return a Err<ErrorKind::SubscriptionNotFound>
//   // fn get_subscription(&mut self, sub_id: & String) -> Result<&mut Subscription> {
//   //   match self.subscriptions.iter_mut().find(|s| & s.sub_id == sub_id) {
//   //     None => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into()),
//   //     Some(sub) => Ok(sub)
//   //   }
//   // }

//   // /// Try and get the serial port
//   // /// If unsuccessful, return a Err<ErrorKind::OpenPortNotFound>
//   // fn get_port(&mut self, port: &String) -> Result<&mut OpenPort> {
//   //   match self.open_ports.get_mut(port) {
//   //     None => Err(ErrorKind::OpenPortNotFound(port.to_string()).into()),
//   //     Some(sp) => Ok(sp)
//   //   }
//   // }

//   /// Open the given port.
//   /// If the port is already open, then just subscribe to it
//   fn open_port(&mut self, sub_id: & String, port_name:& String) -> Result<SerialResponse>  {

//     fn open_serial_port(port:& String) -> Result<Box<sp::SerialPort>>{
//       // If not, open it
//       let sp_settings = sp::SerialPortSettings {
//         baud_rate: sp::BaudRate::Baud115200,
//         data_bits: sp::DataBits::Eight,
//         flow_control: sp::FlowControl::None,
//         parity: sp::Parity::None,
//         stop_bits: sp::StopBits::One,
//         timeout: Duration::from_millis(1),
//       };
//       sp::open_with_settings(&port, &sp_settings).map_err(|err|ErrorKind::SerialportError(err).into())
//     };

//     let sub = self.
//       subscriptions.
//       iter_mut().
//       find(|s| & s.sub_id == sub_id).
//       ok_or(ErrorKind::SubscriptionNotFound(sub_id.to_string()))?;
    
//     let mut sp = try!(open_serial_port(port_name));

//     let port = self.open_ports.get_mut(port_name).or_else(||{
//         let op = OpenPort{write_lock_sub_id: None, port:RefCell::new(sp)};
//         self.open_ports.insert(port_name.to_string(), op);
//         Some(& mut op)
//       });
    
//     match sub.ports.iter().position(|p| p == port_name) {
//       None => {sub.ports.push(port_name.to_string())}
//       _ => {debug!("Port '{}' already subscribed by sub '{}' ", port_name, sub_id)}
//     };
    
//     Ok(SerialResponse::Opened{
//       port: port_name.to_string()
//     })

//     // if subscription.ports.contains(port){
//     //   return 
//     // }

//     // self.get_port(port)?.map_error()


//     // self.get_subscription(sub_id)?.

//     // match self.subscriptions.iter().position(|s| s.sub_id == sub_id){
//     //   // We don't even know who to send an error to...
//     //   None => warn!("No subscription found for sub_id {}",sub_id),
//     //   Some(idx) => {
//     //     let sub = &mut self.subscriptions[idx];
//     //     let serial_port = match self.open_ports.get(&port){
//     //       None => {
//     //         let sp = sp::open_with_settings(&port, &sp_settings);
//     //         self.open_ports.insert(port, sp);
//     //         return
//     //       }
//     //       Some(sp) => Ok(sp.port.borrow_mut())
//     //     };

//     //   }
//     // }

//     // match self.open_ports.get(&port) {
//     //   case Some(serial_port) => {

//     //   },
//     //   case None => {

//     //   }
//     // }

//   }

//   // fn add_sender(&self, port: String, sender: Sender<SerialResponse>) {}

//   fn set_write_lock(&self, sub_id: String, port: String) {}

//   fn release_write_lock(&self, sub_id: String, port: Option<String>) {}

//   fn write_port(&mut self, sub_id: String, port: String, data: String, base64: bool) {
//     // get port

//     // check sub_id matches write_lock id

//     // If it does, try and write

//     // Else send a write lock error
//   }

//   /// Send a response to a subscription with the id sub_id
//   fn send_response(&mut self, sub_id: String, response: SerialResponse) {
//     match self
//             .subscriptions
//             .iter()
//             .position(|s| s.sub_id == sub_id) {
//       Some(idx) => {
//         match self.subscriptions[idx].subscriber.send(response.clone()) {
//           Ok(_) => debug!("Sucessfully sent {:?} to {}", response.clone(), sub_id),
//           Err(_) => {
//             debug!("Remote end for sub_id {} closed, removing", sub_id);
//             self.subscriptions.remove(idx);
//           }
//         }
//       }
//       None => warn!("No subscription found for sub_id '{}'", sub_id),
//     }
//   }

//   /// Broadcast a response to all subscriptions
//   fn broadcast(&mut self, response: SerialResponse) {
//     self
//       .subscriptions
//       .retain(|sub| {
//         sub
//           .subscriber
//           .send(response.clone())
//           .map(|_| true)
//           .unwrap_or_else(|_| {
//                             warn!("Connection closed.");
//                             false
//                           })
//       });
//   }

//   fn close_port(&self, sub_id: String, port: Option<String>) {}



//   /// Fire up the port manager
//   ///
//   /// TODO Right now this is spinning a thread between reading
//   /// subscription requests, reading serial port data
//   /// etc. This will cause high cpu usage
//   ///
//   /// TODO Once this is working, refactor to avoid
//   /// thread spinning?
//   ///
//   fn run(&mut self) {

//     let mut shutdown = false;
//     // Check about 30 times a second
//     let sleep_dur = Duration::from_millis(33);
//     let mut serial_buf: Vec<u8> = vec![0; 4096];

//     while !shutdown {
//       // Sleep for a little bit to avoid pegging cpu
//       thread::sleep(sleep_dur);

//       // Handle serial operation requests
//       match self.receiver.try_recv() {
//         Err(e) => {
//           match e {
//             TryRecvError::Empty => {
//               // nothing to do
//             }
//             TryRecvError::Disconnected => {
//               // Remote end hung up, time to shutdown
//               info!("Shutting down SerialPortManager");
//             }
//           }
//         }
//         Ok(req) => self.handle_serial_request(req),
//       }

//       // Check for new data on each port
//       for (port_name, open_port) in self.open_ports.iter() {
//         // If we have data, send it back
//         // Read needs to borrow &mut self, so wrap port in RC?
//         let result = match open_port
//                 .port
//                 .borrow_mut()
//                 .read(serial_buf.as_mut_slice()) {
//           Ok(bytes_read) => {
//             // We got some data
//             let bytes = &serial_buf[0..bytes_read];
//             // Try and parse the bytes as utf-8
//             match String::from_utf8(bytes.to_vec()) {
//               // We need to send as binary
//               Err(e) => Err(ErrorKind::Utf8Error(e)),
//               Ok(s) => {
//                 Ok(SerialResponse::Read {
//                      port: port_name.to_string(),
//                      data: s,
//                      base64: Some(false),
//                    })
//               }
//             }
//           }
//           Err(_) => Err(ErrorKind::PortReadError(port_name.to_string()).into()),
//         };

//         let r = result.unwrap_or_else(|ek| to_serial_response_error(ek.into()));          

//         // Send report, retain only live connections
//         self
//           .subscriptions
//           .retain(|c| {
//             // If its not the port we are currently interested in, we keep it
//             if !c.ports.contains(&port_name) {
//               true
//             } else {    
//               // Otherwise try and send on it, and if it fails
//               // remove the subscription
//               c.subscriber
//                 .send(r.clone())
//                 .map(|_| true)
//                 .unwrap_or_else(|_| {
//                                   warn!("Connection was closed.");
//                                   false
//                                 })
//             }
//           });
//       }

//       // Check for Subscribe requests
//       match self.subsc_receiver.try_recv() {
//         Err(e) => {
//           match e {
//             TryRecvError::Empty => {
//               // nothing to do
//             }
//             TryRecvError::Disconnected => {
//               // Remote end hung up, time to shutdown
//               shutdown = true;
//               info!("Shutting down SerialPortManager");
//             }
//           }
//         }
//         Ok(sub_req) => {
//           self
//             .subscriptions
//             .push(Subscription {
//                     ports: Vec::with_capacity(4),
//                     subscriber: sub_req.subscriber,
//                     sub_id: sub_req.sub_id,
//                   });
//         }
//       }
//     }
//   }
// }



// // TODO How to spawn and fire it up
// // TODO Document that closing subscription sender will result in
// // portmanager shutdown.
// // fn spawnManager() -> {}

