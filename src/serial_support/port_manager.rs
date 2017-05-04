use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use serialport as sp;
use serial_support::errors::*;


/// Struct for containing Port information
struct OpenPort {
  /// The opened serial port
  /// SerialPort is not Sized, so it makes hashmap mad
  /// and so we deal with these shennanigans
  port: RefCell<Box<sp::SerialPort>>,
}

impl OpenPort {
  
  /// Write data to the serial port
  pub fn write_port(&self, data: &[u8]) -> Result<()> {
    self
      .port
      .borrow_mut()
      .write_all(data)
      .map_err(|err| ErrorKind::Io(err).into())
  }
  /// Read data from the serial port
  pub fn read_port(&self, buff: &mut [u8]) -> Result<usize> {
    self
      .port
      .borrow_mut()
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
    PortManager{
      open_ports: HashMap::new()
    }
  }

  /// Has the port been opened
  fn is_port_open(&self, port_name: &String) -> bool {
    self.open_ports.contains_key(port_name)
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
          let open_port = OpenPort { port: RefCell::new(serial_port) };
          self.open_ports.insert(port_name.to_string(), open_port);
          Ok(())
        }
        Err(e) => Err(ErrorKind::Serialport(e).into()),
      }
    }
  }

  fn close_port(&mut self, port_name: &String) {
    // This drops the underlying serial port and box
    self.open_ports.remove(port_name);
  }

  /// Write data to the port
  /// To write data to a port the port must have been previously locked by sub_id
  fn write_port(&self, port_name: &String, sub_id: &String, data: &[u8]) -> Result<()> {
    match self.open_ports.get(port_name) {
      None => Err(ErrorKind::OpenPortNotFound(port_name.to_string()).into()),
      Some(p) => p.write_port(data),
    }
  }

  /// Read data from a port into the buffer buff
  /// If successful, returns Ok(usize) which is the number of
  /// bytes read
  fn read_port(&self, port_name: &String, buff: &mut [u8]) -> Result<usize> {
    match self.open_ports.get(port_name) {
      None => Err(ErrorKind::OpenPortNotFound(port_name.to_string()).into()),
      Some(p) => p.read_port(buff),
    }
  }

  
  /// Read all currently open ports, return a hashmap of
  /// ports to Result<Vec<u8>>
  pub fn read_all_ports(&self) -> HashMap<String,Result<Vec<u8>>>{
    let mut buffer = vec![0; 4096];
    let mut map = HashMap::new();
      for port_name in self.open_ports.keys(){
        match self.read_port(port_name,buffer.as_mut_slice()){
          Ok(bytes_read) => {
            let bytes = buffer[0..bytes_read].to_vec();
            map.insert(port_name.to_string(), Ok(bytes));
          },
          Err(e) => {
            map.insert(port_name.to_string(), Err(ErrorKind::PortReadError(port_name.to_string()).into()));
          }
        }
      }
    map
  }
}