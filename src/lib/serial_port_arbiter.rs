//! The Serial Port arbiter handles responding to serial port listing commands, restarting Serial //! port actors, and other tasks. It also passes some messages to serial ports.

use crate::errors::*;
use crate::messages::*;
use crate::serial_port_actor::*;

use actix::prelude::*;
use log::*;
use serialport;

use std::collections::HashMap;

#[derive(Debug)]
pub struct SerialPortArbiter {
  // TODO WeakAddr?
  open_ports: HashMap<String, Addr<SerialPortActor>>,
}

impl SerialPortArbiter {
  /// List all serial ports
  pub fn list_ports(&self) -> Result<Vec<serialport::SerialPortInfo>> {
    serialport::available_ports().map_err(|e| ErrorKind::Serialport(e).into())
  }

  /// Launch port actor
  fn launch_port_actor(&mut self, ctx: &mut Context<Self>, serial_port_name: &str) -> () {
    // TODO IF already in map, check if not closed, etc.
    if self.open_ports.get(serial_port_name).is_none() {
      match SerialPortActor::open_port_and_start(ctx.address(), serial_port_name) {
        Ok(address) => {
          self
            .open_ports
            .insert(serial_port_name.to_string(), address);
        }
        Err(error) => {}
      }
    };
  }
}

impl Default for SerialPortArbiter {
  fn default() -> Self {
    SerialPortArbiter {
      open_ports: HashMap::new(),
    }
  }
}

impl Actor for SerialPortArbiter {
  type Context = Context<Self>;
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

      // SerialRequest::Open {} => match self.launch_port_actor() {},
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
