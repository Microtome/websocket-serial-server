extern crate argparse;
extern crate env_logger;
extern crate hyper;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serialport;
extern crate websocket;

mod serial_support;

use std::thread;
use std::io::Write;
use std::collections::HashMap;

use argparse::{ArgumentParser, Store};
use websocket::{Server, Message};
use websocket::message::Type;
use hyper::Server as HttpServer;
use hyper::net::Fresh;
use hyper::server::request::Request;
use hyper::server::response::Response;

use serial_support::messages::*;
use serial_support::common::*;

fn main() {

  // Init logger
  env_logger::init().unwrap();

  // let sr = SerialResponse::Response {
  //     data: "HEY YOU".to_string(),
  //     base64: None,
  // };
  // info!("serialized = {}", serde_json::to_string(&sr).unwrap());

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

  // The HTTP server handler
  let http_handler = move |_: Request, response: Response<Fresh>| {
    let mut response = response.start().unwrap();
    // Send a client webpage
    response
      .write_all(format!(include_str!("websockets.html"), wsPort = port + 1).as_bytes())
      .unwrap();
    response.end().unwrap();
  };

  info!("Using ports {} {}", port, port + 1);

  // Start listening for http connections
  thread::spawn(move || {
                  let http_server = HttpServer::http(format!("127.0.0.1:{}", port)).unwrap();
                  http_server.handle(http_handler).unwrap();
                });

  // Start listening for WebSocket connections
  let ws_server = Server::bind(format!("127.0.0.1:{}", port + 1)).unwrap();

  for connection in ws_server.filter_map(Result::ok) {
    // Spawn a new thread for each connection.
    thread::spawn(move || {
      if !connection
            .protocols()
            .contains(&"rust-websocket".to_string()) {
        connection.reject().unwrap();
        return;
      }

      let mut client = connection
        .use_protocol("rust-websocket")
        .accept()
        .unwrap();

      let ip = client.peer_addr().unwrap();

      info!("Connection from {}", ip);

    //   let message = Message::text("Hello".to_string());
    //   client.send_message(&message).unwrap();

      let (mut receiver, mut sender) = client.split().unwrap();

      for message in receiver.incoming_messages() {
        let message: Message = message.unwrap();

        match message.opcode {

          Type::Close => {
            let message = Message::close();
            sender
              .send_message(&message)
              .unwrap_or_else(|_| info!("Client {} hung up!", ip));
            info!("Client {} disconnected", ip);
            return;
          }

          Type::Ping => {
            let message = Message::pong(message.payload);
            sender
              .send_message(&message)
              .unwrap_or_else(|_| info!("Could not ping client {}!", ip));
          }

          _ => {

            // Get the payload, in a lossy manner
            let msg = String::from_utf8_lossy(&message.payload);

            // Function to remap Json Deserizilation error
            // to a SerialResponse:Error
            let remap_error = |_| -> SerialResponse {
              SerialResponse::Error {
                kind: ErrorType::JsonParseFailure,
                msg: "Unable to parse json".to_string(),
                detail: Some(msg.to_string()),
                port: None,
                handle: None,
              }
            };

            // So we will get a result <SerialRequest::*,SerialResponse::Error> back
            let res: Result<SerialRequest, SerialResponse> = serde_json::from_str(&msg)
              .map_err(remap_error);

            let reply = match res {
              Ok(req) => Message::text(serde_json::to_string(&req).unwrap()),
              Err(err) => Message::text(serde_json::to_string(&err).unwrap()),
            };

            sender
              .send_message(&reply)
              .unwrap_or_else(|_| {
                                info!("Could send message {} to client {}",
                                      String::from_utf8_lossy(&message.payload),
                                      ip)
                              });
          }
        }
      }
    });
  }
}



fn handle_serial_req(req: SerialRequest) {
  match req {
    SerialRequest::Open { port } => {}
    SerialRequest::Write {
      handle,
      port,
      data,
      base64,
    } => {}
    SerialRequest::WriteLock { handle, port } => {}
    SerialRequest::ReleaseWriteLock { handle, port } => {}
  }
}
