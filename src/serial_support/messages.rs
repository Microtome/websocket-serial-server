use serde_json;

// TODO: use when new version drops
//use serialport::{SerialPortInfo,UsbPortInfo,SerialPortType};

use std::fmt;
use std::sync::mpsc::Sender;

#[derive( Clone, Debug)]
pub struct SubscriptionRequest {
  pub sub_id: String,
  pub subscriber: Sender<SerialResponse>,
}

/// Represents the valid json requests that can be made
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SerialRequest {
  // TODO: Divorce subscriptions from port open/close?
  // Right now port is only closed when last subscriber
  // unsubscribes
  /// Open a port for reading
  Open { sub_id: String, port: String },
  /// Take control of a port for writing
  WriteLock { sub_id: String, port: String },
  /// Release control of a port for writing
  /// If no port is given, release all write locks
  /// for all ports
  ReleaseWriteLock {
    sub_id: String,
    port: Option<String>,
  },
  /// Write data, only works if we have control
  Write {
    sub_id: String,
    port: String,
    data: String,
    base64: Option<bool>,
  },
  /// Close a port or subscription
  Close {
    sub_id: String,
    port: Option<String>,
  },
  /// List serial ports
  List { sub_id: String },
}

impl fmt::Display for SerialRequest {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let json = serde_json::to_string(self).unwrap_or("Display SerialRequest: Serialization Failed!"
                                                       .to_string());
    write!(f, "{}", json)
  }
}

/// Represents the valid serial responses that can be made
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SerialResponse {
  /// Error response
  Error {
    description: String,
    display: String,
  },
  /// Data that was read from port
  Read {
    port: String,
    data: String,
    base64: Option<bool>,
  },
  /// Port was closed
  Closed { port: String },
  /// Port was closed
  Opened { port: String },
  /// Command successful
  Ok { msg: String },
  /// List serial ports response
  List { ports: Vec<String> },
}


/*
// TODO: Uncomment when new serialport module version drops
/// Needed for Serde Support as 
/// SerialPortInfo is in seperate module
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(remote = "SerialPortInfo")]
pub struct SerialPortInfoDef {
    /// The short name of the serial port
    pub port_name: String,
    /// The hardware device type that exposes this port
    pub port_type: SerialPortType,

}

#[derive(Serialize, Deserialize)]
#[serde(remote = "sp::SerialPortType")]
/// The physical type of a `SerialPort`
pub enum SerialPortTypeDef {
    /// The serial port is connected via USB
    UsbPort(UsbPortInfoDef),
    /// The serial port is connected via PCI (permanent port)
    PciPort,
    /// The serial port is connected via Bluetooth
    BluetoothPort,
    /// It can't be determined how the serial port is connected
    Unknown,
}


#[derive(Serialize, Deserialize)]
#[serde(remote = "UsbPortInfo")]
pub struct UsbPortInfoDef {
    /// Vender ID
    pub vid: u16,
    /// Product ID
    pub pid: u16,
    /// Serial number (arbitrary string)
    pub serial_number: Option<String>,
    /// Manufacturer (arbitrary string)
    pub manufacturer: Option<String>,
    /// Product name (arbitrary string)
    pub product: Option<String>,
}
*/


impl fmt::Display for SerialResponse {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let json = serde_json::to_string(self).unwrap_or("Display SerialResponse: Serialization Failed!"
                                                       .to_string());
    write!(f, "{}", json)
  }
}
