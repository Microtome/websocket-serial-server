//!
//! # WebSocket Serial Server
//!
//! WebSocket Serial Server is a program that allows that allows browsers to access serial ports on
//! localhost
//!
//! ## Running
//!
//! ```./wsss```
//!
//! For information on configuration please check out the [cfg](../lib/cfg/index.html) package in
//! serialsupport
//!
//! You can open/close ports, read and write data, and see the responses. All messages are in JSON
//! format

#[macro_use]
extern crate log;

use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use hyper::http::{Request, Response};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::Body;
use hyper::Server as HttpServer;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use websocket::message::Type;
use websocket::result::WebSocketError;
use websocket::server::upgrade::WsUpgrade;
use websocket::server::WsServer;
use websocket::Message;

use lib::cfg::*;
use lib::dynamic_sleep::DynamicSleep;
use lib::errors as e;
use lib::manager::Manager;
use lib::messages::*;

/// Max number of failures we allow when trying to send data to client before exiting
/// TODO: Make configurable
pub const MAX_SEND_ERROR_COUNT: u32 = 5;

/// Launches wsss
pub fn main() {
  // Init logger
  env_logger::try_init().expect("Initialization of logging system failed!");

  // Grab config
  let cfg = WsssConfig::load();

  // html file for landing page
  let websocket_html = include_str!("websockets.html").replace(
    "__WS_PORT__ = 8081",
    &format!("__WS_PORT__ = {}", cfg.ws_port),
  );

  // The HTTP server handler
  let http_handler = || {
    service_fn_ok(|_: Request<Body>| {
      // Send a client webpage
      Response::new(Body::from(websocket_html))
    })
  };

  info!("Using ports {} {}", cfg.http_port, cfg.ws_port);

  // Set up channels and Manager
  let (sub_tx, sub_rx) = channel::<SubscriptionRequest>();
  let (sreq_tx, sreq_rx) = channel::<(String, SerialRequest)>();
  Manager::spawn(sreq_rx, sub_rx);

  // Start listening for http connections
  let addr = (cfg.bind_address, cfg.http_port).into();

  let http_server = HttpServer::bind(&addr)
    .serve(http_handler)
    .map_err(|e| eprintln!("server error: {}", e));

  hyper::rt::run(http_server);

  // Start listening for WebSocket connections
  let ws_server = WsServer::bind(format!("{}:{}", cfg.bind_address, cfg.ws_port))
    .expect(&format!("Failed bind on websocket port {}", cfg.ws_port));

  // Continuously iterate over connections, spawning handlers
  for connection in ws_server.filter_map(Result::ok) {
    // Set up subscription id
    // let ts = SystemTime::now() - UNIX_EPOCH
    let prefix: String = thread_rng()
      .sample_iter(&Alphanumeric)
      .take(10)
      .collect::<String>();
    let sub_id = format!("thread-{}-{}", prefix, rand::random::<u16>());
    debug!("{}: spawned.", sub_id);

    // Spawn a new thread for each connection.
    let sub_tx_clone = sub_tx.clone();
    let sreq_tx_clone = sreq_tx.clone();
    spawn_ws_handler(sub_id, sub_tx_clone, sreq_tx_clone, connection);
  }
}

/// Spawn a websocket handler into its own thread
fn spawn_ws_handler(
  sub_id: String,
  sub_tx_clone: Sender<SubscriptionRequest>,
  sreq_tx_clone: Sender<(String, SerialRequest)>,
  connection: WsUpgrade<TcpStream>,
) {
  thread::spawn(move || ws_handler(sub_id, &sub_tx_clone, &sreq_tx_clone, connection));
}

/// Websocket handler
fn ws_handler(
  sub_id: String,
  sub_tx: &Sender<SubscriptionRequest>,
  sreq_tx: &Sender<(String, SerialRequest)>,
  connection: WsUpgrade<TcpStream>,
) {
  if !connection
    .protocols()
    .contains(&"websocket-serial-json".to_string())
  {
    connection.reject().expect(&"Connection rejection failed.");
    return;
  }

  connection
    .tcp_stream()
    .set_nonblocking(true)
    .expect(&"Setting stream non-blocking failed.");

  // Create response channel
  let (sub_resp_tx, sub_resp_rx) = channel::<SerialResponse>();

  // Register sub_id with manager
  sub_tx
    .send(SubscriptionRequest {
      sub_id: sub_id.clone(),
      subscriber: sub_resp_tx,
    })
    .expect(&format!("{}: Registering with manager failed.", sub_id));

  let client = connection
    .use_protocol(format!("websocket-serial-json"))
    .accept()
    .expect(&format!("{}: Accept protocol failed.", sub_id));

  let ip = client
    .peer_addr()
    .expect(&format!("{}: Could not get peer address", sub_id));

  info!("{}: Connection from {}", sub_id, ip);

  let (mut receiver, mut sender) = client
    .split()
    .expect(&format!("{}: WS client error", sub_id));

  let mut send_error_count = 0;

  let mut dynamic_sleep = DynamicSleep::new("main");

  'msg_loop: loop {
    dynamic_sleep.sleep();

    // Try and read a WS message
    match receiver.recv_message::<Message, _, _>() {
      Ok(message) => {
        match message.opcode {
          Type::Close => {
            let message = Message::close();
            sender
              .send_message(&message)
              .unwrap_or(info!("{}: Client {} hung up!", sub_id, ip));
            // Send close request to cleanup resources
            sreq_tx
              .send((sub_id.clone(), SerialRequest::Close { port: None }))
              .unwrap_or_else(|e| {
                warn!(
                  "Client exit cleanup failed for sub_id '{}', cause '{}'",
                  sub_id, e
                )
              });
            info!("{}: Client {} disconnected", sub_id, ip);
            break 'msg_loop;
          }

          Type::Ping => {
            let message = Message::pong(message.payload);
            sender
              .send_message(&message)
              .unwrap_or(info!("{}:  Could not ping client {}!", sub_id, ip));
          }

          _ => {
            // Get the payload, in a lossy manner
            let msg = String::from_utf8_lossy(&message.payload);

            // So we will get a result <SerialRequest::*,SerialResponse::Error> back
            match serde_json::from_str(&msg) {
              Ok(req) => {
                match sreq_tx.send((sub_id.clone(), req)) {
                  Err(err) => {
                    let error = e::ErrorKind::SendRequest(err).into();
                    send_serial_response_error(&sub_id, &mut sender, error);
                  }
                  _ => {}
                };
              }
              Err(err) => {
                let error = e::ErrorKind::Json(err).into();
                send_serial_response_error(&sub_id, &mut sender, error);
              }
            };
          }
        };
      }
      Err(e) => {
        match e {
          WebSocketError::NoDataAvailable => { /*Logging?*/ }
          _ => { /*Logging?*/ }
        };
      }
    }

    // Send on any serial responses
    match sub_resp_rx.try_recv() {
      Ok(resp) => match serde_json::to_string(&resp) {
        Ok(json) => {
          let reply = Message::text(json.clone());
          sender.send_message(&reply).unwrap_or_else(|e| {
            send_error_count += 1;
            info!(
              "{}: Could not send message '{}' to client '{}', cause '{}'",
              sub_id, json, ip, e
            )
          });
        }
        Err(_) => {}
      },
      _ => { /*Logging*/ }
    };

    if send_error_count > MAX_SEND_ERROR_COUNT {
      warn!(
        "{}: Client send error count exceeded! Shutting down msg loop.",
        &sub_id
      );
      break 'msg_loop;
    }
  }

  info!("{}: Shutting down!", sub_id);
}

/// Send an error to the given subscriber Log a warning if the message can't be sent This is usually
/// ok as it means the client has simply disconnected
fn send_serial_response_error(sub_id: &String, sender: &mut Writer<TcpStream>, error: e::Error) {
  let error = e::to_serial_response_error(error);
  serde_json::to_string(&error)
    .map_err(|err| e::ErrorKind::Json(err))
    .map(|json| Message::text(json))
    .map(|msg| {
      sender
        .send_message::<Message, _>(&msg)
        .map_err::<e::Error, _>(|err| e::ErrorKind::SendWsMessage(err).into())
    })
    .unwrap_or_else(|_| {
      warn!("{}: Problem sending bad json error response", sub_id);
      Ok(())
    })
    .is_ok(); // This shouldn't be needed?
}
