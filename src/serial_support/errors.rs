use serial_support::messages::{SerialResponse, SerialRequest};

error_chain! {

  foreign_links {
    Fmt(::std::fmt::Error);
    Io(::std::io::Error) #[cfg(unix)];
    Serialport(::serialport::Error) #[cfg(unix)];
    Utf8(::std::string::FromUtf8Error);
    Json(::serde_json::error::Error);
    SendResponse(::std::sync::mpsc::SendError<SerialResponse>);
    Base64(::base64::DecodeError);
    SendRequest(::std::sync::mpsc::SendError<SerialRequest>);
    SendWsMessage(::websocket::result::WebSocketError);
  }

  errors{
    UnknownRequest{
      description("Unknown request")
      display("Unknown request,")
    }
    OpenPortNotFound(port:String){
      description("Open serial port not found")
      display("Serial port '{}' not found, try opening it first", port)
    }
    SubscriptionNotFound(sub_id:String){
      description("Subscription not found")
      display("Subscription for id '{}' not found", sub_id)
    }
    AlreadyWriteLocked(port:String){
      description("Port already write locked")
      display("Open serial port '{}' is already write locked", port)
    }
    NeedWriteLock(port:String){
      description("Need write lock")
      display("Write to open port '{}' failed, you need to write lock first", port)
    }
    PortReadError(port:String){
      description("Error reading serial port")
      display("Read from port '{}' failed", port)
    }
    PortEOFError(port:String){
      description("Encountered EOF reading serial port")
      display("Encountered EOF reading serial port {}", port)
    }
    PortWriteError(port:String){
      description("Error writing serial port")
      display("Writing to port '{}' failed", port)
    }
    SubscriberSendError(sub_id:String){
      description("Error sending message to subscriber")
      display("Send to subscriber '{}' failed", sub_id)
    }
  }
}

pub fn to_serial_response_error(err: Error) -> SerialResponse {
  SerialResponse::Error {
    description: err.description().to_string(),
    display: format!("{}", err),
  }
}
