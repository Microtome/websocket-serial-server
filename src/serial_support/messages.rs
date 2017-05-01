
/// Represents the valid json requests that can be made
#[derive(Serialize, Deserialize, Clone)]
pub enum SerialRequest {
    /// Open a port for reading
    Open { port: String, req_id: String },
    /// Take control of a port for writing
    WriteLock {
        handle: String,
        port: String,
        req_id: String,
    },
    /// Release control of a port for writing
    ReleaseWriteLock {
        handle: String,
        port: String,
        req_id: String,
    },
    /// Write data, only works if we have control
    Write {
        handle: String,
        port: String,
        data: String,
        base64: Option<bool>,
        req_id: String,
    },
}

/// Represents the possible error types
/// that can be returned
#[derive(Serialize, Deserialize, Clone)]
pub enum ErrorType {
    /// Failed to parse message
    JsonParseFailure,
    /// Unknown request
    UnknownRequest,
    /// Someone else has already locked the port for writing
    AlreadyWriteLocked,
    /// Error reading serial port
    ReadError,
    /// Error writing serial port
    WriteError,
}

/// Represents the valid serial responses that can be made
#[derive(Serialize, Deserialize, Clone)]
pub enum SerialResponse {
    /// Error response
    Error {
        kind: ErrorType,
        msg: String,
        port: Option<String>,
        handle: Option<String>,
    },
    /// Data that was read from port
    Read {
        port: String,
        data: String,
        base64: Option<bool>,
    },
    /// Ok response showing that command was accepted
    Ok { request: SerialRequest },
}
