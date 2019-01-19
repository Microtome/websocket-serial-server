//! The Serial Port arbiter handles responding to serial port listing commands, restarting Serial //! port actors, and other tasks. It also passes messages to/from serial ports.

use crate::messages::{CommandRequest, CommandResponse, SerialResponse};

use actix::prelude::*;
use log::*;

struct SerialPortArbiter {}

impl Actor for SerialPortArbiter {
  type Context = Context<Self>;
}

impl Handler<CommandRequest> for SerialPortArbiter {
  type Result = ();

  fn handle(&mut self, command_request: CommandRequest, ctx: &mut Context<Self>) -> Self::Result {
    println!("Someone joined {:?}", command_request);
    command_request
      .address
      .try_send(CommandResponse {
        address: ctx.address().recipient(),
        response: SerialResponse::Ok {
          msg: "Got Message!".to_string(),
        },
      })
      .unwrap_or_else(|err| debug!("{}", err))
  }
}
