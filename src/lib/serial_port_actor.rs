//! The Serial Port actor handles reading/writing to/from a single serial port.

use crate::errors::*;
use crate::messages::*;
use crate::serial_port_arbiter::*;

use actix::prelude::*;
use log::*;
use serialport;

use std::fmt;
use std::time::Duration;

// Buffer for the port
const SERIAL_PORT_READ_BUFFER_SIZE: usize = 2 ^ 16;

// Scan serial port 30x a second.
static SERIAL_PORT_SCAN_INTERVAL: Duration = Duration::from_millis(33);

// TODO: make settings configurable
static DEFAULT_SERIALPORT_SETTINGS: serialport::SerialPortSettings =
  serialport::SerialPortSettings {
    baud_rate: 115200,
    data_bits: serialport::DataBits::Eight,
    flow_control: serialport::FlowControl::None,
    parity: serialport::Parity::None,
    stop_bits: serialport::StopBits::One,
    timeout: Duration::from_millis(1),
  };

/// Decode data payload.
fn decode_data(data: &str, is_base64: bool) -> Result<Vec<u8>> {
  if is_base64 {
    base64::decode(&data).map_err(|e| ErrorKind::Base64(e).into())
  } else {
    Ok(Vec::from(data.as_bytes()))
  }
}

pub struct SerialPortActor {
  serial_port_name: String,
  serial_port: Box<serialport::SerialPort>,
  arbiter: Addr<SerialPortArbiter>,
  buffer: [u8; SERIAL_PORT_READ_BUFFER_SIZE],
}

impl fmt::Debug for SerialPortActor {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // TODO: Flesh out more...
    write!(
      f,
      "SerialPortActor {{ serial_port_name: {} }}",
      &self.serial_port_name
    )
  }
}

impl SerialPortActor {
  // /// Try and send a response, if it fails, check the error type and perform cleanup.
  // fn do_send(&mut self, recipient: Recipient<CommandResponse>, response: CommandResponse) {
  //   if let Err(error) = recipient.try_send(response) {
  //     debug!(
  //       "Error '{}' occured when trying to send message to {:?}.",
  //       error,
  //       DebugRecipient(&recipient)
  //     );
  //   }
  // }

  /// Read data from the serial port]
  ///
  /// Uses ioctl to check if there are bytes to read. If there are no bytes to read, returns Ok(0).
  /// Returning Ok(0) thus does not indicate EOF, only that there is nothing to read yet.
  /// We check before reading to prevent blocking the thread on the serial fd.
  ///
  /// If there are bytes to read, reads them into the buffer.
  pub fn read_port(&mut self, buff: &mut [u8]) -> Result<usize> {
    let bytes_to_read = self.serial_port.bytes_to_read()?;
    let mut bytes_read = 0;
    if bytes_to_read > 0 {
      bytes_read = self.serial_port.read(&mut self.buffer)?;
    }
    Ok(bytes_read)
  }

  /// Write data to the port managed by this actor.
  pub fn write_port(&mut self, data: &str, is_base64: bool) -> Result<()> {
    decode_data(data, is_base64).and_then(|data| {
      self
        .serial_port
        .write_all(data.as_slice())
        .and_then(|_| self.serial_port.flush())
        .map_err(|err| ErrorKind::Io(err).into())
    })
  }

  /// Static method to fire up a SerialPortActor, bound to a serial port.
  ///
  /// The actor tries to open the port and if successful, enters the running start.
  pub fn open_port_and_start(
    arbiter_address: Addr<SerialPortArbiter>,
    serial_port_name: &str,
  ) -> Result<Addr<SerialPortActor>> {
    serialport::open(serial_port_name)
      .map(|serial_port| {
        SerialPortActor {
          arbiter: arbiter_address,
          buffer: [0; SERIAL_PORT_READ_BUFFER_SIZE],
          serial_port_name: serial_port_name.to_string(),
          serial_port: serial_port,
        }
        .start()
      })
      .map_err(|error| ErrorKind::Serialport(error).into())
  }

  /// Static method used by timer callback to periodically scan ports
  fn read_serial_port(serial_port_actor: &mut SerialPortActor, ctx: &mut Context<Self>) {
    let mut buffer = serial_port_actor.buffer;
    match serial_port_actor.read_port(&mut buffer) {
      Ok(bytes_read) => {
        if bytes_read > 0 {
          let data_read = &serial_port_actor.buffer[0..bytes_read];
          match String::from_utf8(data_read.to_vec()) {
            // We need to send as binary
            Err(error) => Ok(Some(SerialResponse::Read {
              port: serial_port_actor.serial_port_name.clone(),
              data: base64::encode(data_read),
              base64: Some(true),
            })),
            // We can send as ascii
            Ok(utf8_string) => Ok(Some(SerialResponse::Read {
              port: serial_port_actor.serial_port_name.clone(),
              data: utf8_string,
              base64: Some(false),
            })),
          }
        } else {
          Ok(None)
        }
      }
      Err(error) => Err(error),
    }
    .map(|response| match response {
      Some(serial_response) => serial_port_actor.try_arbiter_send_log_failure(serial_response),
      _ => {}
    })
    .map_err(|error| {
      serial_port_actor.try_arbiter_send_log_failure(to_serial_response_error(
        error,
        Some(serial_port_actor.serial_port_name.clone()),
      ))
    })
    .expect("Error reading serial port")
  }

  /// Start scanning the serial port.
  fn start_scan(&self, ctx: &mut Context<Self>) {
    ctx.run_interval(SERIAL_PORT_SCAN_INTERVAL, SerialPortActor::read_serial_port);
  }

  /// Try to send a message to the parent serial port arbiter actor. If the send fails
  /// we log it.
  fn try_arbiter_send_log_failure(&self, serial_response: SerialResponse) {
    if let Err(error) = self.arbiter.try_send(serial_response) {
      debug!(
        "{:?} Encountered error {:?} trying to send to {:?}",
        self, error, self.arbiter
      )
    };
  }
}

impl Actor for SerialPortActor {
  type Context = Context<Self>;

  /// Method is called on actor start.
  /// We register ws session with ChatServer
  fn started(&mut self, ctx: &mut Self::Context) {
    debug!("Starting {:?}", self);
    self.start_scan(ctx)
  }

  /// Method is called on actor stopping.
  /// When actor is dropped it automatically shuts down.
  fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    debug!("Stopping {:?}", self);
    Running::Stop
  }

  /// Method is called on actor stopping.
  /// When actor is dropped it automatically shuts down.
  fn stopped(&mut self, ctx: &mut Self::Context) -> () {
    debug!("Stopped {:?}", self);
  }
}

impl Handler<SerialRequest> for SerialPortActor {
  type Result = ();

  /// Handle serial request messages
  ///
  /// The Serial Port Actor only handles SerialRequest::Write messages, all others will be
  /// logged and ignored.
  fn handle(&mut self, serial_request: SerialRequest, ctx: &mut Context<Self>) -> Self::Result {
    match serial_request {
      SerialRequest::Write { port, data, base64 } => {
        if let Err(write_error) = self.write_port(&data, base64.unwrap_or(false)) {
          self.try_arbiter_send_log_failure(to_serial_response_error(
            write_error,
            Some(self.serial_port_name.clone()),
          ))
        };
      }
      other @ _ => {
        debug!("{:?} should not have gotten message: {:?}", self, other);
      }
    };
  }
}
