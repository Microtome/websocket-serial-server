/// Manages serial port state and communication with clients,
/// and handling requests / responses
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;

use base64;

use crate::common::*;
use crate::dynamic_sleep::DynamicSleep;
use crate::errors::*;
use crate::messages::*;
use crate::port_manager::*;
use crate::sub_manager::*;
use crate::writelock_manager::*;

/// Serial port management module supporting one
/// writer and multiple readers
///
/// Clients can lock a port for writing, but
/// subscribe to data from multiple ports for reads
///
/// The Manager takes actions in response to
/// [SerialRequest::*](../messages/index.html) messages sent on
/// on its receiver
pub struct Manager {
  /// Manage write lock status
  writelock_manager: WriteLockManager,
  /// Manage ports
  port_manager: PortManager,
  /// Manage subscriptions
  sub_manager: SubscriptionManager,
  /// Receiver for serial requests
  receiver: Receiver<(String, SerialRequest)>,
  /// Receiver for response subscription requests
  subsc_receiver: SubscReceiver,
}

impl Manager {
  ///Constructor
  pub fn new(
    receiver: Receiver<(String, SerialRequest)>,
    subsc_receiver: SubscReceiver,
  ) -> Manager {
    Manager {
      writelock_manager: WriteLockManager::new(),
      port_manager: PortManager::new(),
      sub_manager: SubscriptionManager::new(),
      receiver,
      subsc_receiver,
    }
  }

  ///Spawn an instance in a new thread.
  pub fn spawn(
    receiver: Receiver<(String, SerialRequest)>,
    subsc_receiver: SubscReceiver,
  ) -> thread::JoinHandle<()> {
    thread::spawn(move || {
      Manager::new(receiver, subsc_receiver).run();
    })
  }

  /// Main loop
  fn run(&mut self) {
    // Bad ports we couldn't read from
    // A set of SerialResponse::Errors built from
    // from the serial port read/write error responses
    let mut bad_ports = HashSet::<String>::new();

    // Check about 30 times a second
    let mut dynamic_sleep = DynamicSleep::new("manager");

    loop {
      // Sleep for a little bit to avoid pegging cpu
      dynamic_sleep.sleep();

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
              break;
            }
          }
        }
        Ok(req) => self.handle_serial_request(&req.0, req.1),
      }

      // Check for new data on each port
      for (port_name, result) in self.port_manager.read_all_ports() {
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
              },
            };
            self.broadcast_message_for_port(&port_name, response);
          }
          // Send data reads
          Err(e) => {
            warn!("Error reading port!");
            warn!("{}", e);
            bad_ports.insert(port_name);
          }
        }
      }

      //Handle write requests
      let mut recv_count = 0;
      while recv_count < 50 {
        recv_count += 1;
        match self.subsc_receiver.try_recv() {
          Ok(sub_request) => self.sub_manager.add_subscription(sub_request),
          Err(e) => {
            match e {
              TryRecvError::Disconnected => {
                // Does this mean all senders have disconnected?
                // Or just one?
                debug!("Got disconnected when trying to get serial request");
              }
              TryRecvError::Empty => break,
            }
          }
        }
      }

      // Cleanup bad serial ports that failed read or write
      // We remove them from everything before
      self.cleanup_bad_ports(&bad_ports);
      bad_ports.clear();
    }
  }

  /// Handles and dispatches SerialRequest sent by
  /// the channel
  fn handle_serial_request(&mut self, sub_id: &String, msg: SerialRequest) {
    let response = match msg {
      SerialRequest::Open { port } => self.handle_open_port(sub_id, port),
      SerialRequest::WriteLock { port } => self.handle_write_lock(sub_id, port),
      SerialRequest::ReleaseWriteLock { port } => self.handle_release_write_lock(sub_id, port),
      SerialRequest::Write { port, data, base64 } => {
        self.handle_write_port(sub_id, port, data, base64.unwrap_or(false))
      }
      SerialRequest::Close { port } => self.handle_close_port(sub_id, port),
      SerialRequest::List {} => self.handle_list_ports(sub_id),
    };
    if let Err(e) = response {
      warn!("Error '{}' occured handling serial request message", e);
      // Send error?
      self.send_message(&sub_id, e.into());
    }
  }

  /// Handle write port requests
  fn handle_write_port(
    &mut self,
    sub_id: &String,
    port_name: String,
    data: String,
    base_64: bool,
  ) -> Result<()> {
    self.check_sub_id(&sub_id)?;
    self.check_owns_writelock(&port_name, &sub_id)?;
    match base_64 {
      true => base64::decode(&data)
        .map_err(|e| WebsocketSerialServerError::Other(e.into()))
        .and_then(|d| self.port_manager.write_port(&port_name, &d))
        .map(|_| self.send_message(&sub_id, SerialResponse::Wrote { port: port_name })),
      false => self
        .port_manager
        .write_port(&port_name, data.as_bytes())
        .map(|_| self.send_message(&sub_id, SerialResponse::Wrote { port: port_name })),
    }
  }

  /// Handle write lock requests
  fn handle_write_lock(&mut self, sub_id: &String, port_name: String) -> Result<()> {
    self.check_sub_id(&sub_id)?;
    self
      .writelock_manager
      .try_lock_port(&port_name, &sub_id)
      .map(|_| self.send_message(&sub_id, SerialResponse::WriteLocked { port: port_name }))
  }

  /// Handle write requests
  fn handle_release_write_lock(
    &mut self,
    sub_id: &String,
    port_name: Option<String>,
  ) -> Result<()> {
    self.check_sub_id(sub_id)?;
    match port_name {
      None => {
        self.writelock_manager.unlock_all_ports_for_sub(&sub_id);
        Ok(self.send_message(
          &sub_id,
          SerialResponse::WriteLockReleased { port: port_name },
        ))
      }
      Some(port_name) => self
        .writelock_manager
        .unlock_port(&port_name, &sub_id)
        .map(|_| {
          self.send_message(
            &sub_id,
            SerialResponse::WriteLockReleased {
              port: Some(port_name),
            },
          )
        }),
    }
  }

  /// Handle open port requests
  fn handle_open_port(&mut self, sub_id: &String, port_name: String) -> Result<()> {
    self.check_sub_id(&sub_id)?;
    self.port_manager.open_port(&port_name)?;
    self
      .sub_manager
      .add_port(&sub_id, &port_name)
      .map(|_| self.send_message(&sub_id, SerialResponse::Opened { port: port_name }))
  }

  /// Handle list ports request
  fn handle_list_ports(&mut self, sub_id: &String) -> Result<()> {
    self.check_sub_id(&sub_id)?;
    let port_names: Result<Vec<String>> = self
      .port_manager
      .list_ports()
      .map(|v| v.iter().map(|v| v.port_name.clone()).collect());
    match port_names.map(|pns| SerialResponse::List { ports: pns }) {
      Ok(response) => self.send_message(&sub_id, response),
      Err(error) => self.send_message(&sub_id, error.into()),
    }
    Ok(())
  }

  /// Handle close port requests
  fn handle_close_port(&mut self, sub_id: &String, port_name: Option<String>) -> Result<()> {
    match port_name {
      Some(port_name) => self.handle_close_port_for_sub(sub_id, port_name),
      None => self.handle_close_all_ports_for_sub(sub_id),
    }
  }

  /// Handle closing a signle port for a sub
  fn handle_close_port_for_sub(&mut self, sub_id: &String, port_name: String) -> Result<()> {
    self.sub_manager.remove_port(&sub_id, &port_name)?;
    self
      .writelock_manager
      .unlock_port_if_locked_by(&port_name, &sub_id);
    // self.cleanup_ports_with_no_subs();
    let close_resp = SerialResponse::Closed {
      port: port_name.clone(),
    };
    self.send_message(&sub_id, close_resp);
    Ok(())
  }

  /// Handle closing all ports for sub
  fn handle_close_all_ports_for_sub(&mut self, sub_id: &String) -> Result<()> {
    self.sub_manager.clear_ports(Some(&sub_id));
    self.writelock_manager.unlock_all_ports_for_sub(sub_id);

    // Close ports with no subscribers
    let open_ports = self.port_manager.open_ports();
    let subscribed_ports = self.sub_manager.subscribed_ports();
    let ports_with_no_subs = open_ports.difference(&subscribed_ports);

    // For each open port that isn't subscribed,
    for port_to_close in ports_with_no_subs {
      // close it, REDUNDANT?
      self.port_manager.close_port(&port_to_close);
      // remove the write lock, REDUNDANT?
      self.writelock_manager.clear_lock(&port_to_close);
      // Let them know its closed
      let close_resp = SerialResponse::Closed {
        port: port_to_close.clone(),
      };
      self.send_message(&sub_id, close_resp);
    }
    Ok(())
  }

  /// Cleanup any bad ports
  fn cleanup_bad_ports(&mut self, bad_ports: &HashSet<String>) {
    for port_name in bad_ports.iter() {
      // Tell everyone port is sick
      let err_resp: SerialResponse = WebsocketSerialServerError::PortReadError {
        port: port_name.clone(),
      }
      .into();
      self.broadcast_message_for_port(port_name, err_resp);
      // Tell everyone the sick ports were closed
      let close_resp = SerialResponse::Closed {
        port: port_name.clone(),
      };
      self.broadcast_message_for_port(port_name, close_resp);
      // Close bad ports
      self.port_manager.close_port(port_name);
      // Remove write locks on bad ports
      self.writelock_manager.clear_lock(port_name);
      // Remove bad ports from subscriptions
      self.sub_manager.remove_port_from_all(port_name);
    }
  }

  /// Send a message to a subscriber
  fn send_message(&mut self, sub_id: &String, msg: SerialResponse) {
    if let Err(e) = self.sub_manager.send_message(sub_id, msg) {
      warn!("Error sending serial response to sub_id '{}'", sub_id);
      let mut bad_subs = Vec::new();
      bad_subs.push(e);
      self.cleanup_bad_subs(bad_subs);
    }
  }

  /// Broadcast a message to all subscribers and
  /// then cleanup any subs that errored
  fn broadcast_message(&mut self, msg: SerialResponse) {
    let bad_subs = self.sub_manager.broadcast_message(msg);
    self.cleanup_bad_subs(bad_subs);
  }

  /// Broadcast a message to all subscribers interested in the
  /// given port and then cleanup any subs that errored
  fn broadcast_message_for_port(&mut self, port_name: &String, msg: SerialResponse) {
    let bad_subs = self.sub_manager.broadcast_message_for_port(port_name, msg);
    self.cleanup_bad_subs(bad_subs);
  }

  /// Cleanup any bad subs where a message send failed
  fn cleanup_bad_subs(&mut self, bad_subs: Vec<WebsocketSerialServerError>) {
    for e in bad_subs {
      if let WebsocketSerialServerError::SubscriberSendError {
        subscription_id: ref sub_id,
      } = e
      {
        // Remove subscriptions
        self.sub_manager.end_subscription(&sub_id);
        // Remove all write locks held by dead subscription
        self.writelock_manager.unlock_all_ports_for_sub(&sub_id);
      }
    }
  }

  /// Check if a subscription exists for a given sub_id
  fn check_sub_id(&self, sub_id: &String) -> Result<()> {
    self.sub_manager.check_subscription_exists(sub_id)
  }

  /// Check if port_name has a write lock for sub_id
  /// Errors if the port is not writelocked by sub_id
  fn check_owns_writelock(&self, port_name: &String, sub_id: &String) -> Result<()> {
    self
      .writelock_manager
      .check_owns_write_lock(port_name, sub_id)
  }
}
