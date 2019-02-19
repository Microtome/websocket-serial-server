//! The Serial Port actor handles subscriptions and reading/writing to/from a single serial port.

use crate::errors::*;
use crate::messages::*;
use crate::serial_port_arbiter::*;

use actix::prelude::*;
use log::*;
use serialport;

use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

const SERIAL_PORT_READ_BUFFER_SIZE: usize = 2 ^ 16;

// Scan serial port 30x a second.
static SERIAL_PORT_SCAN_INTERVAL: Duration = Duration::from_millis(33);

static DEFAULT_SERIALPORT_SETTINGS: serialport::SerialPortSettings =
  serialport::SerialPortSettings {
    baud_rate: 115200,
    data_bits: serialport::DataBits::Eight,
    flow_control: serialport::FlowControl::None,
    parity: serialport::Parity::None,
    stop_bits: serialport::StopBits::One,
    timeout: Duration::from_millis(1),
  };

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
  subscribers: HashSet<Recipient<CommandResponse>>,
  writelocked: Option<Recipient<CommandResponse>>,
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
  /// Cleanup a dead recipient, removing it from subscriptions and removing its writelock if it
  /// had one.
  fn cleanup_dead_recipient(&mut self, recipient: Recipient<CommandResponse>) {
    self
      .subscribers
      .retain(|subscriber| subscriber != &recipient);
    if Some(recipient) == self.writelocked {
      self.writelocked = None
    }
  }

  /// Try and send a response, if it fails, check the error type and perform cleanup.
  fn do_send(&mut self, recipient: Recipient<CommandResponse>, response: CommandResponse) {
    if let Err(error) = recipient.try_send(response) {
      debug!(
        "Error '{}' occured when trying to send message to {:?}.",
        error,
        DebugRecipient(&recipient)
      );
    }
  }

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

  /// Static method to fire up a SerialPortActor, bound to a serial port Actor.
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
          subscribers: HashSet::new(),
          writelocked: None,
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
            Err(error) => Some(SerialResponse::Read {
              port: serial_port_actor.serial_port_name.clone(),
              data: base64::encode(data_read),
              base64: Some(true),
            }),
            Ok(utf8_string) => Some(SerialResponse::Read {
              port: serial_port_actor.serial_port_name.clone(),
              data: utf8_string,
              base64: Some(false),
            }),
          }
        } else {
          None
        }
      }
      Err(error) => Some(to_serial_response_error(error)),
    }
    .map(|serial_response| {
      let command_response = CommandResponse {
        address: ctx.address().recipient(),
        response: serial_response,
      };
      // TODO Refactor to broadcast message method
      // let bad_recipients: HashSet<Recipient<CommandResponse>> =
      serial_port_actor.subscribers.retain(|recipient| {
        // Don't keep any subscribers who fail send.
        recipient
          .try_send(command_response.clone())
          .map(|_| true)
          .unwrap_or(false)
      });
    });
  }

  fn start_scan(&self, ctx: &mut Context<Self>) {
    ctx.run_interval(SERIAL_PORT_SCAN_INTERVAL, SerialPortActor::read_serial_port);
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

  fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    debug!("Stopping {:?}", self);
    Running::Stop
  }
}

impl Handler<CommandRequest> for SerialPortActor {
  type Result = ();

  fn handle(&mut self, command_request: CommandRequest, ctx: &mut Context<Self>) -> Self::Result {
    let sender = command_request.address;
    match command_request.request {
      SerialRequest::Open { port } => (),
      SerialRequest::Close { port } => (),
      SerialRequest::Write { port, data, base64 } => (),
      SerialRequest::WriteLock { port } => (),
      SerialRequest::ReleaseWriteLock { port } => (),
      SerialRequest::List {} => (),
    }
  }
}
