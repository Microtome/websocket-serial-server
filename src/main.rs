// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate argparse;
extern crate base64;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serialport;
extern crate websocket;

mod serial_support;

use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

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
use rand::{thread_rng, Rng};

use serial_support::manage::Manager;
use serial_support::messages::*;
use serial_support::errors as e;

fn main() {

  // Init logger
  env_logger::init().unwrap();

  let sr = SerialRequest::List {};
  info!("serialized = {}", serde_json::to_string(&sr).unwrap());

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
  let websocket_html = format!(include_str!("websockets.html"), ws_port = ws_port);

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
  thread::spawn(move || {
                  let http_server = HttpServer::http(format!("127.0.0.1:{}", port)).
                  expect(&format!("Failed to create http server on port {}",port));
                  http_server
                    .handle(http_handler)
                    .expect(&"Failed to listen");
                });

  // Start listening for WebSocket connections
  let ws_server = Server::bind(format!("127.0.0.1:{}", ws_port)).
  expect(&format!("Failed bind on websocket port {}",ws_port));

  for connection in ws_server.filter_map(Result::ok) {
    // Spawn a new thread for each connection.
    let sub_tx_clone = sub_tx.clone();
    let sreq_tx_clone = sreq_tx.clone();
    spawn_ws_handler(sub_tx_clone, sreq_tx_clone, connection);
  }
}

fn spawn_ws_handler(sub_tx_clone: Sender<SubscriptionRequest>,
                    sreq_tx_clone: Sender<(String, SerialRequest)>,
                    connection: WsUpgrade<TcpStream>) {
  thread::spawn(move || ws_handler(&sub_tx_clone, &sreq_tx_clone, connection));
}

fn ws_handler(sub_tx: &Sender<SubscriptionRequest>,
              sreq_tx: &Sender<(String, SerialRequest)>,
              connection: WsUpgrade<TcpStream>) {

  // Set up subscription id
  // let ts = SystemTime::now() - UNIX_EPOCH
  let prefix: String = thread_rng().gen_ascii_chars().take(8).collect();
  let sub_id = format!("thread-{}-{}", prefix, rand::random::<u16>());
  debug!("Spawned thread with subId '{}'", sub_id);

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

  //   let message = Message::text("Hello".to_string());
  //   client.send_message(&message).unwrap();

  let (mut receiver, mut sender) = client
    .split()
    .expect(&format!("{}: WS client error", sub_id));

  // TODO Check thread cleanup
  // The new WS Client thread msg loop
  let mut exit = false;

  let sleep_dur = Duration::from_millis(33);

  while !exit {
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
                                warn!("Client exit cleanup failed for sub_id '{}', cause '{}'",
                                      sub_id,
                                      e)
                              });
            info!("{}: Client {} disconnected", sub_id, ip);
            exit = true;
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
            // Weird, but message sending dose work
            // Do I misunderstand unwrap_or?
            sender
              .send_message(&reply)
              .unwrap_or_else(|e| {
                                info!("{}: Could not send message '{}' to client '{}', cause '{}'",
                                      sub_id,
                                      json,
                                      ip,
                                      e)
                              });
          }
          Err(_) => {}
        }
      }
      _ => { /*Logging*/ }
    };
    thread::sleep(sleep_dur);
  }

  info!("Thread {} exiting!", sub_id);

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

}
