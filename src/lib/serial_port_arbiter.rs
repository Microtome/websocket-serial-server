//! The Serial Port arbiter handles responding to serial port listing commands, restarting Serial //! port actors, and other tasks. It also passes messages to/from serial ports.

use crate::errors::*;
use crate::messages::*;

use actix::prelude::*;
use log::*;
use serialport;

pub struct SerialPortArbiter {}

impl Actor for SerialPortArbiter {
  type Context = Context<Self>;
}

impl SerialPortArbiter {
  /// List all serial ports
  pub fn list_ports(&self) -> Result<Vec<serialport::SerialPortInfo>> {
    serialport::available_ports().map_err(|e| ErrorKind::Serialport(e).into())
  }
}

impl Handler<CommandRequest> for SerialPortArbiter {
  type Result = ();

  fn handle(&mut self, command_request: CommandRequest, ctx: &mut Context<Self>) -> Self::Result {
    debug!("Someone joined {:?}", command_request);

    let response = match command_request.request {
      SerialRequest::List {} => match self.list_ports() {
        Ok(ports) => {
          let port_names = ports.into_iter().map(|v| v.port_name.clone()).collect();
          SerialResponse::List { ports: port_names }
        }

        Err(error) => SerialResponse::Error {
          description: error.description().to_string(),
          display: error.to_string(),
        },
      },

      _ => SerialResponse::Ok {
        msg: "Got Message!".to_string(),
      },
    };

    command_request
      .address
      .try_send(CommandResponse {
        address: ctx.address().recipient(),
        response: response,
      })
      .unwrap_or_else(|err| debug!("Error sending Ok: {}", err))
  }
}
