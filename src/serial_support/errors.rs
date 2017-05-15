use messages::{SerialResponse, SerialRequest};

error_chain! {

  foreign_links {
    // Wrapped Format error
    Fmt(::std::fmt::Error);
    // Wrapped IO error
    Io(::std::io::Error) #[cfg(unix)];
    // Wrapped serial port error
    Serialport(::serialport::Error) #[cfg(unix)];
    // Wrapped Ut8 decode error
    Utf8(::std::string::FromUtf8Error);
    // Wrapped json error
    Json(::serde_json::error::Error);
    // Wrapped sync send response error
    SendResponse(::std::sync::mpsc::SendError<SerialResponse>);
    // Wrapped Base64 decode error
    Base64(::base64::DecodeError);
    // Wrapped sync send request error
    SendRequest(::std::sync::mpsc::SendError<(String,SerialRequest)>);
    // wrapped send websocket error.
    SendWsMessage(::websocket::result::WebSocketError);
  }

  errors{
    /// Unknown server request
    UnknownRequest{
      description("Unknown request")
      display("Unknown request,")
    }
    /// Open port not found
    OpenPortNotFound(port:String){
      description("Open serial port not found")
      display("Serial port '{}' not found, try opening it first", port)
    }
    /// Subscription not found
    SubscriptionNotFound(sub_id:String){
      description("Subscription not found")
      display("Subscription for id '{}' not found", sub_id)
    }
    /// Port already write locked error
    AlreadyWriteLocked(port:String){
      description("Port already write locked by another client")
      display("Open serial port '{}' is already write locked by another client", port)
    }
    /// Need write lock error
    NeedWriteLock(port:String){
      description("Need write lock")
      display("Write to open port '{}' failed, you need to write lock first", port)
    }
    /// Serial port read error
    PortReadError(port:String){
      description("Error reading serial port")
      display("Read from port '{}' failed", port)
    }
    /// Serial Port EOF error
    PortEOFError(port:String){
      description("Encountered EOF reading serial port")
      display("Encountered EOF reading serial port {}", port)
    }
    /// Serial port write error
    PortWriteError(port:String){
      description("Error writing serial port")
      display("Writing to port '{}' failed", port)
    }
    /// Send to subscriber error
    SubscriberSendError(sub_id:String){
      description("Error sending message to subscriber")
      display("Send to subscriber '{}' failed", sub_id)
    }
  }
}

/// Convert Error to serial response error enum type
pub fn to_serial_response_error(err: Error) -> SerialResponse {
  SerialResponse::Error {
    description: err.description().to_string(),
    display: format!("{}", err),
  }
}
