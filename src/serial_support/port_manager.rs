use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::iter::FromIterator;
use std::time::Duration;

use serialport as sp;
use errors::*;


/// Struct for containing Port information
struct OpenPort {
  /// The opened serial port
  /// SerialPort is not Sized, so it makes hashmap mad
  /// and so we deal with these shennanigans
  port: Box<sp::SerialPort>,
}

impl OpenPort {
  /// Write data to the serial port
  pub fn write_port(&mut self, data: &[u8]) -> Result<()> {
    self
      .port
      .write_all(data)
      .and_then(|_| self.port.flush())
      .map_err(|err| ErrorKind::Io(err).into())
  }

  /// Read data from the serial port
  pub fn read_port(&mut self, buff: &mut [u8]) -> Result<usize> {
    self
      .port
      .read(buff)
      .map_err(|err| ErrorKind::Io(err).into())
  }
}

/// Manages ports and their locks
pub struct PortManager {
  /// Maintains list of ports
  open_ports: HashMap<String, OpenPort>,
}

impl PortManager {
  /// Create a new PortManager instance
  pub fn new() -> PortManager {
    PortManager { open_ports: HashMap::new() }
  }

  /// Has the port been opened
  pub fn is_port_open(&self, port_name: &String) -> bool {
    self.open_ports.contains_key(port_name)
  }

  /// List all serial ports
  pub fn list_ports(&self) -> Result<Vec<sp::SerialPortInfo>> {
    sp::available_ports().map_err(|e| ErrorKind::Serialport(e).into())
  }

  /// Open a port
  pub fn open_port(&mut self, port_name: &String) -> Result<()> {

    if self.is_port_open(port_name) {
      Ok(())
    } else {

      let sp_settings = sp::SerialPortSettings {
        baud_rate: sp::BaudRate::Baud115200,
        data_bits: sp::DataBits::Eight,
        flow_control: sp::FlowControl::None,
        parity: sp::Parity::None,
        stop_bits: sp::StopBits::One,
        timeout: Duration::from_millis(1),
      };

      match sp::open_with_settings(&port_name, &sp_settings) {
        Ok(serial_port) => {
          let open_port = OpenPort { port: serial_port };
          self.open_ports.insert(port_name.to_string(), open_port);
          Ok(())
        }
        Err(e) => Err(ErrorKind::Serialport(e).into()),
      }
    }
  }

  pub fn close_port(&mut self, port_name: &String) {
    // This drops the underlying serial port and box
    self.open_ports.remove(port_name);
  }

  /// Write data to the port
  pub fn write_port(&mut self, port_name: &String, data: &[u8]) -> Result<()> {
    match self.open_ports.get_mut(port_name) {
      None => Err(ErrorKind::OpenPortNotFound(port_name.to_string()).into()),
      Some(p) => p.write_port(data),
    }
  }

  /// Read data from a port into the buffer buff
  /// If successful, returns Ok(usize) which is the number of
  /// bytes read
  pub fn read_port(&mut self, port_name: &String, buff: &mut [u8]) -> Result<usize> {
    match self.open_ports.get_mut(port_name) {
      None => Err(ErrorKind::OpenPortNotFound(port_name.to_string()).into()),
      Some(p) => p.read_port(buff),
    }
  }


  /// Read all currently open ports, return a hashmap of
  /// ports to Result<Vec<u8>>
  pub fn read_all_ports(&mut self) -> HashMap<String, Result<Vec<u8>>> {
    let mut buffer = vec![0; 4096];
    let mut map = HashMap::new();
    for (port_name, open_port) in self.open_ports.iter_mut() {
      match open_port.read_port(buffer.as_mut_slice()) {
        Ok(bytes_read) => {
          if bytes_read == 0 {
            // EOF
            info!("Received EOF reading from port {}", port_name);
            map.insert(
              port_name.to_string(),
              Err(ErrorKind::PortEOFError(port_name.clone()).into()),
            );
          } else {
            let bytes = buffer[0..bytes_read].to_vec();
            map.insert(port_name.to_string(), Ok(bytes));
          }
        }
        Err(e) => {
          // debug!("Error {} reading from port {}", e, port_name);
          match e.description() {
            "Operation timed out" => {}
            _ => {
              map.insert(port_name.to_string(), Err(e.into()));
            }
          }

        }
      }
    }
    map
  }

  /// Get a vec of open ports
  pub fn open_ports(&self) -> HashSet<String> {
    HashSet::<String>::from_iter(self.open_ports.keys().map(|k| k.clone()))
  }
}

#[cfg(test)]
mod tests {

  use std::io::Write;

  use serialport::SerialPort;
  use serialport::posix::TTYPort;

  use super::*;

  #[test]
  #[cfg(unix)]
  fn test_unix_serialports() {

    let (mut master, mut slave) = TTYPort::pair().expect("Failed to create pseudoterminal pair!");

    slave
      .set_exclusive(false)
      .expect("Failed to set exclusive false");

    let serial_msg = "abcdefg";

    if let Some(s_name) = slave.port_name() {
      let mut port_manager = PortManager::new();

      port_manager
        .open_port(&s_name)
        .expect(&format!("Failed to open slave port {}", s_name));

      master
        .write(serial_msg.as_bytes())
        .expect("Write to master failed!");

      let res = port_manager.read_all_ports();

      assert_eq!(res.len(), 1, "Should have read one port.");

      for (port_name, value) in res {
        match value {
          Ok(bytes) => {
            let read_msg = String::from_utf8_lossy(&bytes);
            assert_eq!(
              serial_msg,
              read_msg,
              "Messages should be same '{}' '{}'",
              serial_msg,
              read_msg
            );
          }
          Err(e) => panic!("Got error reading port {}", e),
        }
      }
    } else {
      panic!("Failed to get slave pty name");
    }
  }

  // TODO: write to slave, read from master


}