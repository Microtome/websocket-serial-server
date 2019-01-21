use actix::prelude::*;
use actix_web::ws;

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
        println!("Websocket Client heartbeat failed, disconnecting!");

        // notify chat server
        // ctx.state().addr.do_send(server::Disconnect { id: act.id });

        // stop actor
        ctx.stop();

        // don't try to send a ping
        return;
      }
      println!("{:?} : PING!", act);
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
    println!("Starting {:?}", self);
    self.start_heartbeat(ctx)
  }

  fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    println!("Stopping {:?}", self);
    Running::Stop
  }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WebsocketClientActor {
  fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
    println!("WEBSOCKET MESSAGE: {:?}", msg);
    match msg {
      ws::Message::Ping(msg) => {
        self.last_heartbeat = Instant::now();
        ctx.pong(&msg);
      }
      ws::Message::Pong(_) => {
        self.last_heartbeat = Instant::now();
      }
      ws::Message::Text(text) => {
        let message = text.trim();
        println!("Got message >> {}", message)
      }
      ws::Message::Binary(bin) => println!("Unexpected binary"),
      ws::Message::Close(_) => {
        ctx.stop();
      }
    }
  }
}
