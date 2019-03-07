//! The Serial Port arbiter handles responding to serial port listing commands, restarting Serial //! port actors, and other tasks. It also passes some messages to serial ports.

use crate::errors::*;
use crate::messages::*;
use crate::serial_port_actor::*;
use crate::websocket_client_actor::WebsocketClientActor;

use actix::prelude::*;
use log::*;
use serialport;

use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
pub struct PortInfo {
  serial_port_name: String,
  serial_port_actor: Addr<SerialPortActor>,
  subscribers: HashSet<Addr<WebsocketClientActor>>,
  writelocked: Option<Addr<WebsocketClientActor>>,
}

impl PortInfo {
  pub fn new(
    serial_port_name: String,
    serial_port_actor: Addr<SerialPortActor>,
    first_subscriber: Addr<WebsocketClientActor>,
  ) -> Self {
    let mut subscribers: HashSet<Addr<WebsocketClientActor>> = HashSet::new();
    subscribers.insert(first_subscriber);
    PortInfo {
      serial_port_name: serial_port_name,
      serial_port_actor: serial_port_actor,
      subscribers: subscribers,
      writelocked: None,
    }
  }
}

#[derive(Debug)]
pub struct SerialPortArbiter {
  open_ports: HashMap<String, PortInfo>,
}

impl SerialPortArbiter {
  fn shutdown_port_actors_with_no_subscribers(&mut self) {
    self.open_ports.retain(|_, port_info| -> bool {
      // Dropped actors clean up their prts
      port_info.subscribers.len() > 0
    });
  }

  /// Cleanup a dead recipient, removing it from subscriptions and removing its writelock if it
  /// had one.
  fn cleanup_dead_recipient(&mut self, recipient: Addr<WebsocketClientActor>) {
    self
      .open_ports
      .values_mut()
      .into_iter()
      .for_each(|mut port_info| {
        port_info
          .subscribers
          .retain(|subscriber| subscriber != &recipient);
        if let Some(recipient) = &port_info.writelocked {
          port_info.writelocked = None;
        };
      });
  }

  /// Broadcast a message to subscribers
  ///
  /// If port is None, broadcast to all subscribers,
  /// otherwise broadcast only to those on the given port.
  fn broadcast_response<S: Into<String>>(
    &mut self,
    ctx: &mut Context<Self>,
    serial_response: &SerialResponse,
    port: Option<S>,
  ) {
    let command_response = CommandResponse {
      address: ctx.address().recipient(),
      response: serial_response.clone(),
    };

    // TODO: Refactor and make prettier later.
    if port.is_some() {
      self
        .open_ports
        .get_mut(&port.unwrap().into())
        .map(|port_info| {
          { &port_info.subscribers }
            .into_iter()
            .for_each(|subscriber| {
              subscriber
                .try_send(command_response.clone())
                .unwrap_or_else(|err| {
                  debug!("Error {:?} broadcasting {:?}", err, command_response)
                });
            });
        });
    } else {
      let mut already_sent: HashSet<&Addr<WebsocketClientActor>> = HashSet::new();

      self.open_ports.values().into_iter().for_each(|port_info| {
        { &port_info.subscribers }
          .into_iter()
          .for_each(|subscriber| {
            if !already_sent.contains(&subscriber) {
              subscriber
                .try_send(command_response.clone())
                .unwrap_or_else(|err| {
                  debug!("Error {:?} broadcasting {:?}", err, command_response)
                });
              already_sent.insert(&subscriber);
            }
          });
      });
    }
  }

  fn shut_down_port_actors_with_no_subscribers(&mut self) {
    // Serial port actors shut down once dropped.
    self
      .open_ports
      .retain(|_, port_info| port_info.subscribers.len() > 0);
  }

  fn handle_writelock_port(&self, port: String) -> Result<SerialResponse> {
    Ok(SerialResponse::Ok {
      msg: "WriteLock Not Implemented".to_string(),
    })
  }

  fn handle_release_writelock_port(&self, port: Option<String>) -> Result<SerialResponse> {
    Ok(SerialResponse::Ok {
      msg: "ReleaseWriteLock Not Implemented".to_string(),
    })
  }

  fn handle_close_port(&self, port: Option<String>) -> Result<SerialResponse> {
    Ok(SerialResponse::Ok {
      msg: "Close Not Implemented".to_string(),
    })
  }

  fn handle_write_port(&self, port: String, data: String, base64: bool) -> Result<SerialResponse> {
    self
      .open_ports
      .get(port.as_str())
      .ok_or(ErrorKind::OpenPortNotFound(port).into())
      .and_then(|port_info| {
        port_info
          .serial_port_actor
          .try_send(SerialRequest::Write {
            port: port_info.serial_port_name.clone(),
            data: data.clone(),
            base64: Some(base64),
          })
          .map(|_| SerialResponse::Wrote {
            port: port_info.serial_port_name.clone(),
            data: data.clone(),
            base64: Some(base64),
          })
          .map_err(|send_error| to_send_serial_request_error(send_error))
      })
  }

  fn handle_open_port(
    &mut self,
    port: String,
    websocket_actor_address: Addr<WebsocketClientActor>,
    ctx: &mut Context<Self>,
  ) -> Result<SerialResponse> {
    match self.open_ports.contains_key(&port) {
      true => Ok(SerialResponse::Opened { port: port }),
      false => {
        // port needs to be opened.
        match SerialPortActor::open_port_and_start(ctx.address(), &port) {
          Ok(serial_port_actor_address) => {
            self.open_ports.insert(
              port.clone(),
              PortInfo::new(
                port.clone(),
                serial_port_actor_address,
                websocket_actor_address.clone(),
              ),
            );
            Ok(SerialResponse::Opened { port: port.clone() })
          }
          Err(error) => Err(error),
        }
      }
    }
  }

  /// List all serial ports
  fn list_ports(&self) -> Result<Vec<serialport::SerialPortInfo>> {
    serialport::available_ports().map_err(|e| ErrorKind::Serialport(e).into())
  }

  fn handle_list_ports(&self) -> Result<SerialResponse> {
    match self.list_ports() {
      Ok(ports) => {
        let port_names = ports.into_iter().map(|v| v.port_name.clone()).collect();
        Ok(SerialResponse::List { ports: port_names })
      }

      Err(error) => Err(error),
    }
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

impl Handler<SerialResponse> for SerialPortArbiter {
  type Result = ();

  /// Handle serial responses generated by SerialPortActors that are reading ports
  /// These will have their address rewritten and then forwarded on to all
  /// subscribers.
  fn handle(&mut self, serial_response: SerialResponse, ctx: &mut Context<Self>) -> Self::Result {
    debug!("Got port read! {:?}", serial_response);
    debug!("Subscriptions: {:?}", self.open_ports);
    match serial_response {
      SerialResponse::Read { ref port, .. } => {
        self.broadcast_response(ctx, &serial_response, Some(port.to_string()))
      }
      SerialResponse::Error { ref port, .. } => {
        self.broadcast_response(ctx, &serial_response, port.clone().map(|p| p.to_string()))
      }
      other @ _ => {
        // We shouldn't get these...
        debug!(
          "We should not have gotten {:?} from serial port actor",
          other
        )
      }
    }
  }
}

impl Handler<CommandRequest> for SerialPortArbiter {
  type Result = ();

  fn handle(&mut self, command_request: CommandRequest, ctx: &mut Context<Self>) -> Self::Result {
    debug!("Got message: {:?}", command_request);

    let response_address = command_request.address;

    match command_request.request {
      SerialRequest::List {} => self.handle_list_ports(),

      SerialRequest::Write { port, data, base64 } => {
        self.handle_write_port(port, data, base64.unwrap_or(false))
      }

      SerialRequest::Close { port } => self.handle_close_port(port),

      SerialRequest::WriteLock { port } => self.handle_writelock_port(port),

      SerialRequest::ReleaseWriteLock { port } => self.handle_release_writelock_port(port),

      SerialRequest::Open { port } => self.handle_open_port(port, response_address.clone(), ctx),
    }
    .and_then(|response| {
      // TODO: Migrate to failure crate and fix.
      response_address
        .try_send(CommandResponse {
          address: ctx.address().recipient(),
          response: response,
        })
        .map_err(|send_error| to_send_command_response_error(send_error))
    })
    .unwrap_or_else(|error| debug!("Error sending Ok: {:?}", error));
  }
}
