//!
//! # WebSocket Serial Server
//!
//! WebSocket Serial Server is a program that allows browsers to access serial ports on localhost.
//!
//! ## Running
//!
//! ```./wsss```
//!
//! For information on configuration please check out the [cfg](../lib/cfg/index.html)
//! package in lib/.
//!
//! You can open/close ports, read and write data, and see the responses. All messages are in JSON
//! format.

extern crate actix;
extern crate actix_web;
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

extern crate lib;

use actix_web::{http, server, App, HttpRequest, HttpResponse};

use lib::cfg::*;

/// Max number of failures we allow when trying to send
/// data to client before exiting
/// TODO: Make configurable
pub const MAX_SEND_ERROR_COUNT: u32 = 5;

// HTML Template
const WEBSOCKET_HTML_TEMPLATE: &str = include_str!("./websockets.html");

fn index_handler(request: &HttpRequest<String>) -> HttpResponse {
  HttpResponse::Ok()
    .content_type("text/html")
    .body(request.state())
}

/// Launches wsss
pub fn main() {
  // Init logger
  env_logger::init().expect("Initialization of logging system failed!");

  // Grab config
  let cfg = WsssConfig::load();

  info!("Using ports {} {}", cfg.http_port, cfg.ws_port);

  // html file for landing page
  let websocket_html = WEBSOCKET_HTML_TEMPLATE.replace(
    "__WS_PORT__ = 8081",
    &format!("__WS_PORT__ = {}", cfg.ws_port),
  );

  let system = actix::System::new("wsss");

  server::new(move || {
    vec![
      // Index html
      App::with_state(websocket_html.clone())
        .prefix("/")
        .resource("/", |r| r.method(http::Method::GET).f(index_handler))
        .resource("/index.html", |r| {
          r.method(http::Method::GET).f(index_handler)
        })
        .finish(),
    ]
  })
  .bind(format!("{}:{}", cfg.bind_address, cfg.http_port))
  .expect(
    format!(
      "Cannot start server on {}:{}",
      cfg.bind_address, cfg.http_port
    )
    .as_str(),
  )
  .shutdown_timeout(15)
  .start();

  let _ = system.run();
}
