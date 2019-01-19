//! This module contains all of the enums used
//! to represent messages that can be send by clients
//! ( SerialRequest::* ) and their responses by
//! by the server ( SerialReponse::* )

// TODO: use when new version drops
//use serialport::{SerialPortInfo,UsbPortInfo,SerialPortType};

use serde_json;

use actix::prelude::*;

use std::fmt;

#[derive(Clone, Message)]
pub struct CommandRequest {
  pub address: Recipient<CommandResponse>,
  pub request: SerialRequest,
}

impl fmt::Debug for CommandRequest {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "CommandResponse {{ address_ptr: {:p} request: {} }}",
      &self.address, self.request
    )
  }
}

#[derive(Clone, Message)]
pub struct CommandResponse {
  pub address: Recipient<CommandRequest>,
  pub response: SerialResponse,
}

impl fmt::Debug for CommandResponse {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "CommandResponse {{ address_ptr: {:p} request: {} }}",
      &self.address, self.response
    )
  }
}

// impl Message for CommandResponse {
//   type Result = ();
// }

/// Represents the valid json requests that can be made
///
/// On the server side, every client is associated with
/// a unique subscription id which is used
/// to associate a given connection with their
/// operations
///
/// Requests that fail or can not be met will result
/// in SerialResponse::Error responses
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SerialRequest {
  /// Open a port for reading
  ///
  /// Opening the same port more than once is
  /// okay and has no ill effects
  ///
  ///``` json
  /// JSON:
  /// {"Open":{"port":"/dev/ttyUSB"}}
  ///```
  Open { port: String },
  /// Take control of a port for writing
  ///
  /// ``` json
  /// JSON:
  /// {"WriteLock":{"port":"/dev/ttyUSB"}}
  /// ```
  WriteLock { port: String },
  /// Release control of a port for writing
  /// If no port is given, release all write locks
  /// held by the client for all ports
  ///
  /// ``` json
  /// JSON:
  /// {"ReleaseWriteLock":{"port":"/dev/ttyUSB"}}
  ///
  /// {"ReleaseWriteLock":{}}
  /// ```
  ReleaseWriteLock { port: Option<String> },
  /// Write data, only works if the client
  /// has a WriteLock active for the given port
  ///
  /// The base64 property is only required
  /// if the data is encoded as base64
  ///
  /// ``` json
  /// JSON:
  /// {"Write":{"port":"/dev/ttyUSB",
  ///           "data": "Hello World"
  ///          }}
  ///
  /// {"Write":{"port":"/dev/ttyUSB",
  ///           "data": "SGVsbG8gV29ybGQ=",
  ///           "base64": true
  ///          }}
  /// ```
  Write {
    port: String,
    data: String,
    base64: Option<bool>,
  },
  /// Close the port, which stops any read updates
  /// from the port from being sent to this subscription
  ///
  /// If no port is specified, then all ports are 'closed'
  ///
  /// In most cases this behaves more as an 'unsubscribe'
  /// as the hardware port is only closed and released
  /// when the last client connected to it sends a Close message
  ///
  /// ``` json
  /// JSON:
  /// {"Close":{"port":"/dev/ttyUSB"}}
  ///
  /// {"Close":{}}
  /// ```
  Close { port: Option<String> },
  /// List available serial ports
  ///
  /// ``` json
  /// JSON:
  /// {"List":{}}
  /// ```
  List {},
}

impl fmt::Display for SerialRequest {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let json = serde_json::to_string(self)
      .unwrap_or("Display SerialRequest: Serialization Failed!".to_string());
    write!(f, "{}", json)
  }
}

/// Represents the valid json responses that can be made
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SerialResponse {
  /// Error response
  ///
  /// ``` json
  /// JSON:
  /// {"Error":{
  ///            "description":"Error reading serial port",
  ///            "display":"Error reading serial port '/dev/ttyUSB0'"
  ///          }}
  ///
  /// ```
  Error {
    description: String,
    display: String,
  },
  /// Data that was read from port
  ///
  /// If the data could not be parsed into a utf8 string
  /// then it is base64 encoded and the base64
  /// property is set to true.
  ///
  /// ``` json
  /// JSON:
  /// {"Read":{"port":"/dev/ttyUSB",
  ///           "data": "Hello World"
  ///          }}
  ///
  /// {"Read":{"port":"/dev/ttyUSB",
  ///           "data": "SGVsbG8gV29ybGQ=",
  ///           "base64": true
  ///          }}
  /// ```
  Read {
    port: String,
    data: String,
    base64: Option<bool>,
  },
  /// Port was closed
  ///
  /// Sent in response to SerialRequest::Close
  /// or sent when the server detects that a serial
  /// port is misbehaving, closes it, and then
  /// notifies the clients
  ///
  ///``` json
  /// JSON:
  /// {"Closed":{"port":"/dev/ttyUSB"}}
  ///```
  Closed { port: String },
  /// Port was opened
  ///
  /// Sent in response to SerialRequest::Open
  ///
  ///``` json
  /// JSON:
  /// {"Opened":{"port":"/dev/ttyUSB"}}
  ///```
  Opened { port: String },
  /// Command successful
  Ok { msg: String },
  /// Wrote data
  ///
  /// Sent in response to SerialReques::Write
  ///
  /// Notifies the client that the request to write
  /// data was received, and the data was successfully
  /// written to the port specified  
  ///
  /// ``` json
  /// JSON:
  /// {"Wrote":{"port":"/dev/ttyUSB"}}
  /// ```
  ///
  /// TODO: Return hash of data written?
  Wrote { port: String },
  /// Port successfully writelocked
  ///
  /// Sent in response to SerialReques::WriteLock
  ///
  /// Notifies the client that the WriteLock was received
  /// and completed successfully
  ///
  /// ``` json
  /// JSON:
  /// {"WriteLock":{"port":"/dev/ttyUSB"}}
  /// ```
  WriteLocked { port: String },
  /// WriteLocks on Port(s) successfully released
  ///
  /// Sent in response to SerialReques::ReleaseWriteLock
  ///
  /// Notifies the client that the ReleaseWriteLock was received
  /// and completed successfully
  ///
  /// If ReleaseWrite was sent without a port specified, then
  /// it will be empty in the response. This means that all
  /// write locks held on behalf of the client were released.
  ///
  /// ``` json
  /// JSON:
  /// {"WriteLockReleased":{"port":"/dev/ttyUSB"}}
  ///
  /// {"WriteLockReleased":{}}
  /// ```
  WriteLockReleased { port: Option<String> },
  /// List serial ports response
  ///
  /// ``` json
  /// JSON:
  /// {"List":{"ports":["/dev/ttyUSB0","/dev/ttyUSB1"]}}
  /// ```
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
    let json = serde_json::to_string(self)
      .unwrap_or("Display SerialResponse: Serialization Failed!".to_string());
    write!(f, "{}", json)
  }
}
