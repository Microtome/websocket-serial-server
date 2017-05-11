use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;

use serial_support::errors::*;
use serial_support::messages::*;



/// Subscription
struct Subscription {
  /// Subscription
  subscriber: Sender<SerialResponse>,
  /// The ports it is subscribed to
  ports: Vec<String>,
}

impl Subscription {
  /// Send a message to a subscriber
  fn send_message(&self, msg: SerialResponse) -> Result<()> {
    self
      .subscriber
      .send(msg)
      .map_err(|e| ErrorKind::SendResponse(e).into())
  }

  // Register interest in a port
  fn add_port(&mut self, port_name: &String) {
    match self.ports.iter().position(|p| p == port_name) {
      None => self.ports.push(port_name.to_string()),
      Some(_) => debug!("Port already subscribed to"),
    }
  }

  // Remove interest in a port
  fn remove_port(&mut self, port_name: &String) {
    self.ports.retain(|p| *p != *port_name);
  }

  // Remove interest in a port
  fn remove_all_ports(&mut self) {
    self.ports.clear();
  }
}

/// The subscription manager maintains a registry of
/// subscriptions vs ports
/// and provides methods for sending
/// messages to subscribers
pub struct SubscriptionManager {
  /// The subscriptions
  subscriptions: HashMap<String, Subscription>,
}

impl SubscriptionManager {
  /// Create a new SubscriptionManager instance
  pub fn new() -> SubscriptionManager {
    SubscriptionManager { subscriptions: HashMap::new() }
  }

  /// Add a port to a subscription
  pub fn add_port(&mut self, sub_id: &String, port_name: &String) -> Result<()> {
    match self.subscriptions.get_mut(sub_id) {
      Some(sub) => {
        sub.add_port(port_name);
        Ok(())
      }
      None => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into()),
    }
  }


  /// Remove a single port from a given subscription or all subscriptions
  pub fn remove_port(&mut self, sub_id: &String, port_name: &String) -> Result<()> {
    match self.subscriptions.get_mut(sub_id) {
      Some(sub) => {
        sub.ports.retain(|p| p != port_name);
        Ok(())
      }
      None => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into()),
    }
  }

  /// Remove a port from all subscriptions
  pub fn remove_port_from_all(&mut self, port_name: &String) {
    for (_, sub) in self.subscriptions.iter_mut() {
      sub.ports.retain(|p| p != port_name);
    }
  }

  /// Remove all ports from a single subscription or all subscriptions
  pub fn clear_ports(&mut self, sub_id: Option<&String>) {
    match sub_id {
      Some(sid) => {
        match self.subscriptions.get_mut(sid) {
          Some(sub) => {
            sub.ports.clear();
          }
          None => {}
        }
      }
      None => {
        for (_, sub) in self.subscriptions.iter_mut() {
          sub.ports.clear();
        }
      }
    }
  }

  /// Add a subscription
  pub fn add_subscription(&mut self, sub: SubscriptionRequest) {
    self
      .subscriptions
      .entry(sub.sub_id)
      .or_insert(
        Subscription {
          subscriber: sub.subscriber,
          ports: Vec::new(),
        },
      );
  }

  /// Check if subscription exists, otherwise return error
  pub fn check_subscription_exists(&self, sub_id: &String) -> Result<()> {
    match self.subscriptions.contains_key(sub_id) {
      true => Ok(()),
      false => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into()),
    }
  }

  /// End a subscription
  pub fn end_subscription(&mut self, sub_id: &String) {
    self.subscriptions.remove(sub_id);
  }

  /// Send a message to the given subscription
  pub fn send_message(&self, sub_id: &String, msg: SerialResponse) -> Result<()> {
    match self.subscriptions.get(sub_id) {
      None => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into()),
      Some(sub) => {
        sub
          .subscriber
          .send(msg)
          .map_err(|e| ErrorKind::SendResponse(e).into())
      }
    }
  }

  /// Broadcast a messages to all subscribers, returning
  /// a vec of (sub_id,ErrorKind::SendResponse) failures if some sends fail
  pub fn broadcast_message(&self, msg: SerialResponse) -> Vec<Error> {
    debug!("Broadcasting '{}' to all subscribers", &msg);
    let mut res = Vec::new();
    for (sub_id, sub) in self.subscriptions.iter() {
      match self.send_message(sub_id, msg.clone()) {
        Err(e) => res.push(e),
        _ => debug!("  Broadcast to '{}' was successful", sub_id),
      }
    }
    res
  }

  /// Broadcast message to all subscribers registered for a given port
  pub fn broadcast_message_for_port(&self, port_name: &String, msg: SerialResponse) -> Vec<Error> {
    debug!(
      "Broadcasting '{}' to all subscribers registered on port {}",
      &msg,
      port_name
    );
    let mut res = Vec::new();
    for (sub_id, sub) in self.subscriptions.iter() {
      if sub
           .ports
           .iter()
           .position(|p| *p == *port_name)
           .is_some() {
        match self.send_message(sub_id, msg.clone()) {
          Err(e) => res.push(e),
          _ => debug!("  Broadcast to '{}' was successful", sub_id),
        }
      }
    }
    res
  }

  /// Get a list of ports that currently have subscriptions
  pub fn subscribed_ports(&mut self) -> HashSet<String> {
    let mut subscribed_ports = HashSet::<String>::new();
    for subs in self.subscriptions.values() {
      subscribed_ports.extend(subs.ports.clone());
    }
    subscribed_ports
  }
}
