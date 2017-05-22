//! Loads and centralizes configuration from
//! the command line, env, and config files
//!
//! For information on command line switches, config files,
//! or env names, check the documentation for Config
//!

use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::net::Ipv4Addr;
use std::str::FromStr;

use config::{Config, Environment, File, Value};

use argparse::{ArgumentParser, StoreOption};

/// Default HTTP port to bind to if none given
pub const DEFAULT_HTTP_PORT: u32 = 10080;
/// Default Websocket port to bind to if none given
pub const DEFAULT_WS_PORT: u32 = 10081;
/// Default ip address to bind
pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1";

/// Suported config file extensions
pub const SUPPORTED_EXTENSIONS: &[&str] = &["yaml", "yml", "json", "toml"];
/// Config file base name
pub const BASE_CONFIG_FILE_NAME: &str = "wsss_conf";

pub const CONF_FILE_ENV_KEY: &str = "WSSS_CONF_FILE";
pub const BIND_ADDRESS_ENV_KEY: &str = "WSSS_BIND_ADDRESS";
pub const HTTP_PORT_ENV_KEY: &str = "WSSS_HTTP_PORT";
pub const WS_PORT_ENV_KEY: &str = "WSSS_WS_PORT";

const HTTP_PORT_KEY: &str = "http_port";
const WS_PORT_KEY: &str = "ws_port";
const BIND_ADDRESS_KEY: &str = "bind_address";


/// Config is a struct storing global
/// configuration information derived
/// from commandline, file, and env
/// variables
///
/// Sample json config:
/// ``` json
/// {
///   "http_port": 8080,
///   "ws_port": 8082,
///   "bind_address": "10.1.100.12"
/// }
/// ```
///
/// Sample toml config:
/// ``` toml
///   http_port = 8080
///   ws_port = 8082
///   bind_address = "10.1.100.12"
/// ```
///
/// Sample yaml config:
/// ``` yml
///   http_port: 8080
///   ws_port: 8082
///   bind_address: "10.1.100.12"
/// ```
pub struct WsssConfig {
  /// The http_port to listen on.
  ///
  /// Defaults to 10080
  ///
  /// env var WSSS_HTTP_PORT
  ///
  /// cmdline switch -p or --http_port
  pub http_port: u32,

  /// The ws port to listen on.
  ///
  /// defaults to http_port + 1
  ///
  /// env var WSSS_WS_PORT
  ///
  /// cmdline switch -w or --ws_port
  pub ws_port: u32,

  /// Address to bind to.
  ///
  /// Defaults to 127.0.0.1 (localhost)
  ///
  /// env var WSSS_BIND_ADDR
  ///
  /// cmdline -a or --bind_address
  pub bind_address: Ipv4Addr,
}

impl WsssConfig {
  /// Try and load configuration from several well known sources
  /// command line arguments, and env vars
  ///
  /// TODO: First we try and load a json/toml/yml config file
  /// specified by the environment variable WSSS_CONF_FILE.
  ///
  /// TODO: If not found, we then try loading /etc/wsss/wsss.(json|toml|yml).
  ///
  /// TODO: If not found we then try and load a wsss.(json|toml|yml) from the
  /// directory wsss was launched from.
  ///
  /// Then for any settings loaded from these files, we override them
  /// with any env vars we find, then override with any commandline
  /// parameters
  pub fn load() -> WsssConfig {


    let mut cfg = Config::new();
    // Set Defaults
    cfg
      .set_default(HTTP_PORT_KEY, DEFAULT_HTTP_PORT as i64)
      .expect("Failed to set default HTTP port");
    cfg
      .set_default(WS_PORT_KEY, DEFAULT_WS_PORT as i64)
      .expect("Failed to set default WS port");
    cfg
      .set_default(BIND_ADDRESS_KEY, DEFAULT_BIND_ADDR)
      .expect("Failed to set default bind address");

    // look for file under WSSS_CONF_FILE
    // let env_cfg = Config::new();
    // look for toml/yml/json file in /etc
    // look for toml/yml/json file in local dir
    
    // Get env args
    cfg
      .merge(Environment::new("WSSS"))
      .expect("Failed to set values from env");
    
    // get commandline args
    parse_cmdline(&mut cfg);

    WsssConfig {
      http_port: cfg.get_int("http_port").map(|i| i as u32).unwrap(),
      ws_port: cfg.get_int("ws_port").map(|i| i as u32).unwrap(),
      bind_address: Ipv4Addr::from_str(&cfg.get_str("bind_address").unwrap()).unwrap(),
    }
  }
}

impl Default for WsssConfig {
  /// Create a WsssConfig with all values
  /// set to default
  fn default() -> WsssConfig {
    WsssConfig {
      http_port: DEFAULT_HTTP_PORT,
      ws_port: DEFAULT_WS_PORT,
      bind_address: Ipv4Addr::from_str(DEFAULT_BIND_ADDR).unwrap(),
    }
  }
}

/// Parse the command line returning a config with
/// defaults overridden by commandline values.
fn parse_cmdline(cfg: &mut Config) {

  let mut port: Option<u32> = None;
  let mut ws_port: Option<u32> = None;
  let mut bind_address: Option<String> = None;

  {
    let mut ap = ArgumentParser::new();
    ap.set_description("Provide access to serial ports over JSON Websockets");
    ap.refer(&mut port)
      .add_option(&["-p", "--http_port"], StoreOption, "Http Port");
    ap.refer(&mut ws_port)
      .add_option(&["-w", "--ws_port"], StoreOption, "Websocket Port");
    ap.refer(&mut bind_address)
      .add_option(&["-a", "--bind_address"], StoreOption, "Bind Address");
    ap.parse_args_or_exit();
  }

  port.map(|p| cfg.set(HTTP_PORT_KEY, p as i64));
  ws_port.map(|w| cfg.set(WS_PORT_KEY, w as i64));
  port.map(|a| cfg.set(BIND_ADDRESS_KEY, a as i64));
}
