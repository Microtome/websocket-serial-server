use std::collections::HashMap;

use crate::errors::*;

/// Manages tracking of write locks
pub struct WriteLockManager {
  /// Map of port to subscription ids
  write_locks: HashMap<String, String>,
}

impl WriteLockManager {
  /// Create a new WriteLockManager instance
  pub fn new() -> WriteLockManager {
    WriteLockManager {
      write_locks: HashMap::new(),
    }
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

  /// Is the port locked by someone else
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
      None => Err(WebsocketSerialServerError::AlreadyWriteLocked {
        port: port_name.to_owned(),
      }),
      Some(sid) => {
        if sid != sub_id {
          Err(WebsocketSerialServerError::AlreadyWriteLocked {
            port: port_name.to_owned(),
          })
        } else {
          Ok(())
        }
      }
    }
  }

  /// Clear a write lock, without checking subscriber id
  pub fn clear_lock(&mut self, port_name: &String) {
    self.write_locks.remove(port_name);
  }

  /// Release the write lock for the given port and sub id
  pub fn unlock_port(&mut self, port_name: &String, sub_id: &String) -> Result<()> {
    match self.is_port_locked_by_someone_else(port_name, sub_id) {
      false => {
        self.write_locks.remove(port_name);
        Ok(())
      }
      true => Err(WebsocketSerialServerError::AlreadyWriteLocked {
        port: port_name.to_owned(),
      }),
    }
  }

  /// If port port_name is locked by sub_id, unlock it
  pub fn unlock_port_if_locked_by(&mut self, port_name: &String, sub_id: &String) {
    if self.is_port_write_locked_by(port_name, sub_id) {
      // Should not panic since we are the one who locked it
      self.unlock_port(&port_name, sub_id).unwrap();
    }
  }

  /// Release all write locks held by this sub_id
  pub fn unlock_all_ports_for_sub(&mut self, sub_id: &String) {
    let mut to_delete = Vec::<String>::new();
    for port_name in self.write_locks.keys() {
      if self
        .write_locks
        .get(port_name)
        .map(|sid| sid == sub_id)
        .unwrap_or(false)
      {
        to_delete.push(port_name.to_string());
      }
    }
    for delete_port in to_delete.iter() {
      self.write_locks.remove(delete_port);
    }
  }

  /// Try and lock the port
  pub fn try_lock_port(&mut self, port_name: &String, sub_id: &String) -> Result<()> {
    match self.is_port_locked_by_someone_else(port_name, sub_id) {
      false => {
        self
          .write_locks
          .insert(port_name.to_string(), sub_id.to_string());
        Ok(())
      }
      true => Err(WebsocketSerialServerError::AlreadyWriteLocked {
        port: port_name.to_owned(),
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_locking() {
    /// Utility method
    fn check_not_locked_by_anyone(
      wl_manager: &WriteLockManager,
      port: &String,
      sub_ids: Vec<&String>,
    ) {
      for sub_id in sub_ids.iter() {
        assert_eq!(
          false,
          wl_manager.is_port_write_locked_by(port, sub_id),
          "Port '{}' should not be locked by '{}'",
          port,
          sub_id
        );
        assert_eq!(
          false,
          wl_manager.is_port_locked_by_someone_else(port, sub_id),
          "Port '{}' should not be locked by someone else",
          port
        );
      }
    }

    /// Utility method
    fn check_locked_by_sub(
      wl_manager: &WriteLockManager,
      port: &String,
      sub_locker: &String,
      sub_ids: Vec<&String>,
    ) {
      assert_eq!(
        true,
        wl_manager.is_port_write_locked_by(port, sub_locker),
        "Port '{}' should not be locked by '{}'",
        port,
        sub_locker
      );
      for sub_id in sub_ids.iter() {
        assert_eq!(
          true,
          wl_manager.is_port_locked_by_someone_else(port, sub_id),
          "Port '{}' should not be locked by someone else",
          port
        );
      }
    }

    let wl_manager = &mut WriteLockManager::new();
    let sub_id1: String = "SUB_ID1".to_owned();
    let sub_id3: String = "SUB_ID2".to_owned();
    let sub_id2: String = "SUB_ID3".to_owned();
    let port: String = "/dev/TTY_USB".to_owned();
    // Ports should not be locked
    check_not_locked_by_anyone(&wl_manager, &port, vec![&sub_id1, &sub_id2, &sub_id3]);
    // sub_id1 locking a port should work
    assert_eq!(
      true,
      wl_manager
        .try_lock_port(&port, &sub_id1)
        .map(|_| true)
        .unwrap_or(false),
      "Sub_id '{}' locking port '{}' should succeed",
      sub_id1,
      port
    );
    //Port should now be locked by sub_id1
    check_locked_by_sub(&wl_manager, &port, &sub_id1, vec![&sub_id2, &sub_id3]);
    // sub_id2 should fail locking port already locked
    assert_eq!(
      true,
      wl_manager
        .try_lock_port(&port, &sub_id2)
        .map(|_| false)
        .unwrap_or(true),
      "Sub_id '{}' locking port '{}' should fail",
      sub_id1,
      port
    );
    // sub_id2 should fail unlocking port locked by sub_id1
    assert_eq!(
      true,
      wl_manager
        .unlock_port(&port, &sub_id2)
        .map(|_| false)
        .unwrap_or(true),
      "Sub_id '{}' unlocking port '{}' should fail",
      sub_id1,
      port
    );
    // sub_id1 should be able to unlock it
    assert_eq!(
      true,
      wl_manager
        .unlock_port(&port, &sub_id1)
        .map(|_| true)
        .unwrap_or(false),
      "Sub_id '{}' unlocking port '{}' should succeed",
      sub_id1,
      port
    );
    // Ports should not be locked
    check_not_locked_by_anyone(&wl_manager, &port, vec![&sub_id1, &sub_id2, &sub_id3]);
    // TODO: Finish testing all other methods
  }
}
