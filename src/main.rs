//!
//! # WebSocket Serial Server
//!
//! WebSocket Serial Server is a program that allows
//! that allows browsers to access serial ports
//! on localhost
//!
//! ## Running
//!
//! ```./wsss```
//!
//! You can also specift a port:
//!
//! ```./wsss -p PORT_NUM``` or ```./wsss --port PORT_NUM```
//!
//! When you specify a port, a simple http server will
//! be bound at "/" on PORT_NUM, and a websocket bound at "/" 
//! on PORT_NUM + 1
//! 
//! The webpage served at "/" on PORT_NUM provides a convenient
//! way for testing serial port connectivity
//!
//! You can open/close ports, read and write data, and see the
//! responses. All messages are in JSON format

extern crate argparse;
extern crate base64;
extern crate env_logger;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate rand;
extern crate serde_json;
extern crate serialport;
extern crate websocket;

extern crate serial_support;

use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use argparse::{ArgumentParser, Store};
use websocket::client::Writer;
use websocket::result::WebSocketError;
use websocket::message::Type;
use websocket::{Server, Message};
use websocket::server::upgrade::WsUpgrade;
use hyper::Server as HttpServer;
use hyper::net::Fresh;
use hyper::server::request::Request;
use hyper::server::response::Response;
use rand::{Rng, thread_rng};

use serial_support::dynamic_sleep::DynamicSleep;
use serial_support::errors as e;
use serial_support::manage::Manager;
use serial_support::messages::*;


/// Max number of failures we allow when trying to send
/// data to client before exiting
/// TODO: Make configurable
pub const MAX_SEND_ERROR_COUNT: u32 = 5;

/// Launches wsss
pub fn main() {

  // Init logger
  env_logger::init().unwrap();

  // Default port number
  let mut port = 8080;
  // Parse cmdline args
  {
    let mut ap = ArgumentParser::new();
    ap.set_description("Provide access to serial ports over JSON Websockets");
    ap.refer(&mut port)
      .add_option(&["-p", "--port"], Store, "Http Port");
    ap.parse_args_or_exit();
  }

  // websocket port
  let ws_port = port + 1;

  // html file for landing page
  let websocket_html = include_str!("websockets.html").replace(
    "__WS_PORT__ = 8081",
    &format!("__WS_PORT__ = {}", ws_port),
  );

  // The HTTP server handler
  let http_handler = move |_: Request, response: Response<Fresh>| {
    let mut response = response.start().expect(&"Could not start response");
    // Send a client webpage
    response
      .write_all(websocket_html.as_bytes())
      .expect(&"Could not get template as bytes");
    response.end().expect(&"Send response failed");
  };

  info!("Using ports {} {}", port, ws_port);

  // Set up channels and Manager
  let (sub_tx, sub_rx) = channel::<SubscriptionRequest>();
  let (sreq_tx, sreq_rx) = channel::<(String, SerialRequest)>();
  Manager::spawn(sreq_rx, sub_rx);

  // Start listening for http connections
  let http_server = HttpServer::http(format!("127.0.0.1:{}", port)).expect(
    &format!(
      "Failed to create http server on port {}",
      port
    ),
  );

  thread::spawn(
    move || {
      http_server
        .handle(http_handler)
        .expect(&"Failed to listen");
    },
  );

  // Start listening for WebSocket connections
  let ws_server = Server::bind(format!("127.0.0.1:{}", ws_port)).expect(
    &format!(
      "Failed bind on websocket port {}",
      ws_port
    ),
  );

  // Continuously iterate over connections,
  // spawning handlers
  for connection in ws_server.filter_map(Result::ok) {
    // Set up subscription id
    // let ts = SystemTime::now() - UNIX_EPOCH
    let prefix: String = thread_rng().gen_ascii_chars().take(8).collect();
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
        .contains(&"websocket-serial-json".to_string()) {
    connection
      .reject()
      .expect(&"Connection rejection failed.");
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
    .send(
      SubscriptionRequest {
        sub_id: sub_id.clone(),
        subscriber: sub_resp_tx,
      },
    )
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
              .unwrap_or_else(
                |e| {
                  warn!(
                    "Client exit cleanup failed for sub_id '{}', cause '{}'",
                    sub_id,
                    e
                  )
                },
              );
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
      Ok(resp) => {
        match serde_json::to_string(&resp) {
          Ok(json) => {
            let reply = Message::text(json.clone());
            sender
              .send_message(&reply)
              .unwrap_or_else(
                |e| {
                  send_error_count += 1;
                  info!(
                    "{}: Could not send message '{}' to client '{}', cause '{}'",
                    sub_id,
                    json,
                    ip,
                    e
                  )
                },
              );
          }
          Err(_) => {}
        }
      }
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

/// Send an error to the given subscriber 
/// Log a warning if the message can't be sent
/// This is usually ok as it means the client 
/// has simply disconnected
fn send_serial_response_error(sub_id: &String, sender: &mut Writer<TcpStream>, error: e::Error) {
  let error = e::to_serial_response_error(error);
  serde_json::to_string(&error)
    .map_err(|err| e::ErrorKind::Json(err))
    .map(|json| Message::text(json))
    .map(
      |msg| {
        sender
          .send_message::<Message, _>(&msg)
          .map_err::<e::Error, _>(|err| e::ErrorKind::SendWsMessage(err).into())
      },
    )
    .unwrap_or_else(
      |_| {
        warn!("{}: Problem sending bad json error response", sub_id);
        Ok(())
      },
    )
    .is_ok(); // This shouldn't be needed?

}
