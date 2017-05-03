extern crate serde_json;

use std::sync::mpsc::{Sender};
// use serial_support::errors::SerialResponseError;

pub struct SubscriptionRequest {
  pub sub_id: String,
  pub subscriber: Sender<SerialResponse> 
}

/// Represents the valid json requests that can be made
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SerialRequest {
  /// Open a port for reading
  Open { sub_id: String, port: String},
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
  /// Close a port
  Close {  sub_id: String, port: Option<String> },
}

/// Represents the possible error types
/// that can be returned
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ErrorType {
  /// Port not found
  PortNotFound,
  /// Subscription Id not found
  SubscriptionNotFound,
  /// Failed to parse message
  JsonParseFailure,
  /// Unknown request
  UnknownRequest,
  /// Someone else has already locked the port for writing
  AlreadyWriteLocked,
  /// WriteLock needed
  NeedWriteLock,
  /// Error reading serial port
  ReadError,
  /// Error writing serial port
  WriteError,
}

/// Represents the valid serial responses that can be made
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SerialResponse {
  /// Error response
  Error {
    description: String,
    display: String
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
  /// Ok response showing that command was accepted
  Accepted { request: SerialRequest },
}
