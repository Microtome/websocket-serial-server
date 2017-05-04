use std::collections::HashMap;
use std::sync::mpsc::{Sender};

use serial_support::errors::*;
use serial_support::messages::*;



/// Subscription
struct Subscription {
  /// Subscription
  subscriber: Sender<SerialResponse>,
  /// The ports it is subscribed to
  ports: Vec<String>,
}

impl Subscription{

  /// Send a message to a subscriber
  fn send_message(&self, msg: SerialResponse) -> Result<()>{
    self.subscriber.send(msg).map_err(|e|ErrorKind::SendResponse(e).into())
  }

  // Register interest in a port
  fn add_port(&mut self, port_name: &String){
    match self.ports.iter().position(|p| p==port_name){
      None => self.ports.push(port_name.to_string()),
      Some(_) => debug!("Port already subscribed to")
    }
  }

  // Remove interest in a port
  fn remove_port(&mut self, port_name: &String){
    self.ports.retain(|p| *p != *port_name);
  }
  
  // Remove interest in a port
  fn remove_all_ports(&mut self){
    self.ports.clear();
  }
}

/// Subscription Manager
pub struct SubscriptionManager{
  /// The subscriptions
  /// Maintains list of ports
  subscriptions: HashMap<String, Subscription>,
}

impl SubscriptionManager{

  /// Create a new SubscriptionManager instance
  pub fn new() -> SubscriptionManager {
    SubscriptionManager{
      subscriptions: HashMap::new()
    }
  }

  fn add_port(&mut self, sub_id: &String, port_name: &String) -> Result<()>{
    match self.subscriptions.get_mut(sub_id) {
      Some(sub) => {
        sub.add_port(port_name);
        Ok(())
      }
      None => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into())
    }
  }

  fn remove_port(&mut self, sub_id: &String, port_name: &String) -> Result<()>{
    match self.subscriptions.get_mut(sub_id) {
      Some(sub) => {
        sub.remove_port(port_name);
        Ok(())
      }
      None => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into())
    }
  }

  fn remove_all_ports(&mut self, sub_id: &String, port_name: &String)-> Result<()>{
    match self.subscriptions.get_mut(sub_id) {
      Some(sub) => {
        sub.remove_all_ports();
        Ok(())
      }
      None => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into())
    }
  }

  fn end_subscription(&mut self, sub_id: &String){
    self.subscriptions.remove(sub_id);
  }

  fn send_message(&self, sub_id: &String, msg: SerialResponse) -> Result<()>{
    match self.subscriptions.get(sub_id) {
      None => Err(ErrorKind::SubscriptionNotFound(sub_id.to_string()).into()),
      Some(sub) => sub.subscriber.send(msg).map_err(|e|ErrorKind::SendResponse(e).into())
    }
  }

  /// Broadcast a messages to all subscribers, returning
  /// a vec of ErrorKind::SendResponse failures if some sends fail
  fn broadcast_message(&self, sub_id: &String, msg: SerialResponse) -> Vec<Error>{
    debug!("Broadcasting '{}' to all subscribers", &msg);
    let mut res = Vec::new();
    for(sub_id, sub) in self.subscriptions.iter(){
      match self.send_message(sub_id, msg.clone()){
        Err(e) => res.push(e),
        _ => {debug!("  Broadcast to '{}' was successful", sub_id)}
      }
    }
    res
  }

}
