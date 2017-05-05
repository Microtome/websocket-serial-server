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
    WriteLockManager { write_locks: HashMap::new() }
  }

  /// Is the port write locked by the given sub_id
  pub fn is_port_write_locked_by(&self, port_name: &String, sub_id: &String) -> bool {
    match self.write_locks.get(port_name) {
      None => false,
      Some(sid) => sid == sub_id,
    }
  }

  /// Is the port write locked at all
  pub fn is_port_write_locked(&self, port_name: &String) -> bool {
    self.write_locks.get(port_name).is_some()
  }

  pub fn is_port_locked_by_someone_else(&self, port_name: &String, sub_id: &String) -> bool {
    match self.write_locks.get(port_name) {
      None => false,
      Some(sid) => sid != sub_id,
    }
  }

  /// Check if sub id has write lock on this port, if it doesn't,
  /// return error
  pub fn check_owns_write_lock(&self, port_name: &String, sub_id: &String) -> Result<()> {
    match self.write_locks.get(port_name) {
      None => Ok(()),
      Some(sid) => {
        if sid != sub_id {
          Err(ErrorKind::AlreadyWriteLocked(port_name.to_string()).into())
        } else {
          Ok(())
        }
      }
    }
  }

  // Clear a write lock, without checking subscriber id
  pub fn clear_lock(&mut self, port_name: &String) {
    self.write_locks.remove(port_name);
  }

  // Release the write lock for the given port and sub id
  pub fn unlock_port(&mut self, port_name: &String, sub_id: &String) -> Result<()> {
    self.check_owns_write_lock(port_name, sub_id)?;
    self.write_locks.remove(port_name);
    Ok(())
  }

  // Release the write lock for the given port and sub id
  pub fn unlock_all_ports_for_sub(&mut self, sub_id: &String) {
    let mut to_delete = Vec::<String>::new();
    for port_name in self.write_locks.keys() {
      if self
           .write_locks
           .get(port_name)
           .map(|sid| sid == sub_id)
           .unwrap_or(false) {
        to_delete.push(port_name.to_string());
      }
    }
    for delete_port in to_delete.iter() {
      self.write_locks.remove(delete_port);
    }
  }

  // Try and lock the port
  pub fn lock_port(&mut self, port_name: &String, sub_id: &String) -> Result<()> {
    self.check_owns_write_lock(port_name, sub_id)?;
    self
      .write_locks
      .insert(port_name.to_string(), sub_id.to_string());
    Ok(())
  }
}
