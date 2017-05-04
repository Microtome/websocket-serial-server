use std::collections::HashMap;

use serial_support::errors::*;

/// Manages tracking of write locks
pub struct WriteLockManager {
  /// Map of port to subscription ids
  write_locks: HashMap<String, String>,
}

impl WriteLockManager {

  /// Create a new WriteLockManager instance
  pub fn new() -> WriteLockManager {
    WriteLockManager{
      write_locks: HashMap::new()
    }
  }

  /// Is the port write locked by the given sub_id
  fn is_port_write_locked_by(&self, port_name: &String, sub_id: &String) -> bool {
    match self.write_locks.get(port_name) {
      None => false,
      Some(sid) => sid == sub_id,
    }
  }

  /// Is the port write locked at all
  fn is_port_write_locked(&self, port_name: &String) -> bool {
    self.write_locks.get(port_name).is_some()
  }

  fn is_port_locked_by_someone_else(&self, port_name: &String, sub_id: &String) -> bool {
    match self.write_locks.get(port_name) {
      None => false,
      Some(sid) => sid != sub_id,
    }
  }

  // Release the write lock for the given port and sub id
  fn release_write_lock(&mut self, port_name: &String, sub_id: &String) {
    self.write_locks.remove(port_name);
  }

  // Try and lock the port
  fn lock_port(&mut self, port_name: &String, sub_id: &String) -> Result<()> {
    let locked = self.is_port_locked_by_someone_else(port_name, sub_id);
    if locked {
      Err(ErrorKind::AlreadyWriteLocked(port_name.to_string()).into())
    } else {
      self
        .write_locks
        .insert(port_name.to_string(), sub_id.to_string());
      Ok(())
    }
  }
}
