use actix::prelude::*;
use actix_web::ws;
use log::*;

use crate::messages::*;
use crate::serial_port_arbiter::SerialPortArbiter;

use std::time::{Duration, Instant};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WebsocketClientActor {
  last_heartbeat: Instant,
}

impl WebsocketClientActor {
  fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self, WebsocketClientState>) {
    ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
      // check client heartbeats
      if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
        // heartbeat timed out
        warn!("Websocket Client heartbeat failed, disconnecting!");

        // notify chat server
        // ctx.state().addr.do_send(server::Disconnect { id: act.id });

        // stop actor
        ctx.stop();

        // don't try to send a ping
        return;
      }
      debug!("{:?} : PING!", act);
      ctx.ping("PING!");
    });
  }
}

impl Default for WebsocketClientActor {
  fn default() -> Self {
    WebsocketClientActor {
      last_heartbeat: Instant::now(),
    }
  }
}

pub struct WebsocketClientState {
  pub serial_port_arbiter_address: Addr<SerialPortArbiter>,
}

impl Actor for WebsocketClientActor {
  type Context = ws::WebsocketContext<Self, WebsocketClientState>;

  /// Method is called on actor start.
  /// We register ws session with ChatServer
  fn started(&mut self, ctx: &mut Self::Context) {
    debug!("Starting {:?}", self);
    self.start_heartbeat(ctx)
  }

  fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    debug!("Stopping {:?}", self);
    Running::Stop
  }
}

impl Handler<CommandResponse> for WebsocketClientActor {
  type Result = ();

  fn handle(&mut self, command_response: CommandResponse, ctx: &mut Self::Context) -> Self::Result {
    match serde_json::to_string(&command_response.response) {
      Ok(response) => ctx.text(response),
      Err(err) => warn!("Error serializing response!"),
    };
  }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebsocketClientActor {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    debug!("WEBSOCKET MESSAGE: {:?}", msg);
    match msg {
      ws::Message::Ping(msg) => {
        self.last_heartbeat = Instant::now();
        ctx.pong(&msg);
      }
      ws::Message::Pong(_) => {
        self.last_heartbeat = Instant::now();
      }
      ws::Message::Text(text) => {
        debug!("Got message >> {}", text.trim());
        match serde_json::from_str::<SerialRequest>(&text) {
          Ok(serial_request) => ctx
            .state()
            .serial_port_arbiter_address
            .do_send(CommandRequest {
              address: ctx.address().recipient::<CommandResponse>(),
              request: SerialRequest::List {},
            }),
          Err(err) => warn!("Error reading message '{}' from web socket!", &text),
        }
      }
      ws::Message::Binary(bin) => warn!("Unexpected binary"),
      ws::Message::Close(_) => {
        ctx.stop();
      }
    }
  }
}
