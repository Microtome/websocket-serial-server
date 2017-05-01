extern crate serde_json;

/// Represents the valid json requests that can be made
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SerialRequest {
  /// Open a port for reading
  Open { port: String },
  /// Take control of a port for writing
  WriteLock { handle: String, port: String },
  /// Release control of a port for writing
  /// If no port is given, release all write locks
  /// for all ports
  ReleaseWriteLock {
    handle: String,
    port: Option<String>,
  },
  /// Write data, only works if we have control
  Write {
    handle: String,
    port: String,
    data: String,
    base64: Option<bool>,
  },
  /// Close a port
  Close { port: String },
}

/// Represents the possible error types
/// that can be returned
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ErrorType {
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
    kind: ErrorType,
    msg: String,
    detail: Option<String>,
    port: Option<String>,
    handle: Option<String>,
  },
  /// Data that was read from port
  Read {
    port: String,
    data: String,
    base64: Option<bool>,
  },
  /// Port was closed
  Closed { port: String },
  /// Ok response showing that command was accepted
  Accepted { request: SerialRequest },
}

// json_ser_safely
