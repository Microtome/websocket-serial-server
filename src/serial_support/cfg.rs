//! Loads and centralizes configuration from
//! config files, env, and command line
//!
//! For information on command line switches, config files,
//! or env names, check the documentation for [WsssConfig](struct.WsssConfig.html)

use std::convert::Into;
use std::default::Default;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::Ipv4Addr;
use std::str::FromStr;

use argparse::{ArgumentParser, StoreOption};
use errors::*;
use toml;

/// Default HTTP port to bind to if none given
pub const DEFAULT_HTTP_PORT: u32 = 10080;
/// Default Websocket port to bind to if none given
pub const DEFAULT_WS_PORT: u32 = 10081;
/// Default ip address to bind
pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1";

/// Suported config file extensions
pub const SUPPORTED_EXTENSIONS: &[&str] = &["toml"];
/// Config file base name
pub const CONFIG_FILE_NAME: &str = "wsss_conf.toml";

/// Env variable name for specifying a config file
pub const CONF_FILE_ENV_KEY: &str = "WSSS_CONF_FILE";
/// Env variable name for specifying address to bind
pub const BIND_ADDRESS_ENV_KEY: &str = "WSSS_BIND_ADDRESS";
/// Env variable name for specifying HTTP port
pub const HTTP_PORT_ENV_KEY: &str = "WSSS_HTTP_PORT";
/// Env variable name for specifying WS port
pub const WS_PORT_ENV_KEY: &str = "WSSS_WS_PORT";

const HTTP_PORT_KEY: &str = "http_port";
const WS_PORT_KEY: &str = "ws_port";
const BIND_ADDRESS_KEY: &str = "bind_address";

/// This struct models partially specified configuration,
/// leaving unset values as None
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TomlWsssConfig {
  pub http_port: Option<u32>,
  pub ws_port: Option<u32>,
  pub bind_address: Option<String>,
}

impl TomlWsssConfig {
  /// Convert to a WsssConfig with default values
  /// substituted for missing values
  pub fn to_config(self) -> Result<WsssConfig> {

    let addr_string: String = self
      .bind_address
      .unwrap_or(DEFAULT_BIND_ADDR.to_string());

    let ip_addr = Ipv4Addr::from_str(&addr_string)?;
    // .map_err(|e| ErrorKind::IpAddr(e).into())?;

    Ok(
      WsssConfig {
        http_port: self.http_port.unwrap_or(DEFAULT_HTTP_PORT),
        ws_port: self.ws_port.unwrap_or(DEFAULT_WS_PORT),
        bind_address: ip_addr,
      },
    )
  }

  /// Merge partial configuration read from different sources
  pub fn merge<T: Into<TomlWsssConfig>>(self, other: T) -> TomlWsssConfig {
    let o = other.into();
    TomlWsssConfig {
      http_port: merge_options(self.http_port, o.http_port),
      ws_port: merge_options(self.ws_port, o.ws_port),
      bind_address: merge_options(self.bind_address, o.bind_address),
    }
  }

  /// Parse the command line returning a config with
  /// defaults overridden by commandline values.
  pub fn parse_cmdline() -> TomlWsssConfig {

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

    TomlWsssConfig {
      http_port: port,
      ws_port: ws_port,
      bind_address: bind_address,
    }
  }

  /// Parse a file to yield a partial config
  pub fn parse_file(file_name: &str) -> Result<TomlWsssConfig> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let toml_cfg: TomlWsssConfig = toml::from_str(&contents)?;
    Ok(toml_cfg)
  }

  /// Save partial config to a file
  pub fn save_to_file(&self, file_name: &str) -> Result<()> {
    let mut file = File::open(file_name)?;
    let cfg_toml = toml::to_string(self)?;
    file.write_all(cfg_toml.as_bytes())?;
    file.flush()?;
    Ok(())
  }

  /// Parse configuration out of env variables.
  /// See the cfg::*_ENV_KEY values
  pub fn parse_env() -> TomlWsssConfig {
    TomlWsssConfig {
      http_port: env::var(HTTP_PORT_ENV_KEY)
        .ok()
        .and_then(|v| v.parse::<u32>().ok()),
      ws_port: env::var(WS_PORT_ENV_KEY)
        .ok()
        .and_then(|v| v.parse::<u32>().ok()),
      bind_address: env::var(BIND_ADDRESS_ENV_KEY).ok(),
    }
  }
}

/// Convert a WsssConfig to a TomlWsssConfig
impl From<WsssConfig> for TomlWsssConfig {
  fn from(wsss_cfg: WsssConfig) -> TomlWsssConfig {
    TomlWsssConfig {
      http_port: Some(wsss_cfg.http_port),
      ws_port: Some(wsss_cfg.ws_port),
      bind_address: Some(wsss_cfg.bind_address.to_string()),
    }
  }
}

/// Merge o2 into o1, only if o2 is missing a value for a particular key
fn merge_options<T>(o1: Option<T>, o2: Option<T>) -> Option<T> {
  match (o1, o2) {
    (None, Some(v2)) => Some(v2),
    (None, None) => None,
    (Some(v1), _) => Some(v1),
  }
}

/// Config is a struct storing global
/// configuration information derived
/// from commandline, file, and env
/// variables
///
/// Sample toml config:
///
/// ``` toml
///   http_port = 8080
///   ws_port = 8082
///   bind_address = "10.1.100.12"
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
  /// First we try and load a toml config file
  /// specified by the environment variable `WSSS_CONF_FILE`.
  ///
  /// If not found, we then try loading `/etc/wsss/wsss_conf.toml`.
  ///
  /// If not found we then try and load a wsss_conf.toml from the
  /// directory wsss was launched from.
  ///
  /// Only the contents of the first file found will be loaded
  /// and then modified by env and commandline variables
  ///
  /// Then for any settings loaded from the file, we override them
  /// with any env vars we find, then override with any commandline
  /// parameters
  pub fn load() -> WsssConfig {

    let file_cfg = load_env_file()
      .or_else(|| load_etc())
      .or_else(|| load_local_file())
      .unwrap_or(TomlWsssConfig::default());

    let env_cfg = TomlWsssConfig::parse_env();

    let cmdline_cfg = TomlWsssConfig::parse_cmdline();

    return TomlWsssConfig::merge(
      cmdline_cfg,
      TomlWsssConfig::merge(
        env_cfg,
        TomlWsssConfig::merge(file_cfg, WsssConfig::default()),
      ),
    )
               .into();
  }

  /// Save the configuration to a file
  pub fn save_to_file(&self, file_name: &str) -> Result<()> {
    let mut file = File::open(file_name)?;
    let cfg_toml = toml::to_string(self)?;
    file.write_all(cfg_toml.as_bytes())?;
    file.flush()?;
    Ok(())
  }
}

/// Provide a default configuration
/// filled out with default values
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

/// Convert a TomlWsssConfig to a fully filled out
/// WsssConfig replacing None with default values
impl From<TomlWsssConfig> for WsssConfig {
  fn from(toml_wsss_cfg: TomlWsssConfig) -> WsssConfig {

    let addr_string: String = toml_wsss_cfg
      .bind_address
      .unwrap_or(DEFAULT_BIND_ADDR.to_string());

    let ip_addr = Ipv4Addr::from_str(&addr_string).unwrap();

    WsssConfig {
      http_port: toml_wsss_cfg.http_port.unwrap_or(DEFAULT_HTTP_PORT),
      ws_port: toml_wsss_cfg.ws_port.unwrap_or(DEFAULT_WS_PORT),
      bind_address: ip_addr,
    }
  }
}

/// Try a load config from /etc on unices
#[cfg(unix)]
fn load_etc() -> Option<TomlWsssConfig> {
  TomlWsssConfig::parse_file(&format!("/etc/wsss/{}", CONFIG_FILE_NAME)).ok()
}

/// Dummy method for loading etc config on windows
#[cfg(not(unix))]
fn load_etc() -> Option<TomlWsssConfig> {
  None
}

/// Try and and load a config file
/// specified by the env variable cfg::CONF_FILE_ENV_KEY
fn load_env_file() -> Option<TomlWsssConfig> {
  env::var(CONF_FILE_ENV_KEY)
    .ok()
    .and_then(|file_name| TomlWsssConfig::parse_file(&file_name).ok())
}

/// Try and load config from same directory as executable
fn load_local_file() -> Option<TomlWsssConfig> {
  env::current_exe()
    .ok()
    .and_then(
      |mut dir| {
        dir.pop();
        Some(dir)
      },
    )
    .and_then(|file| TomlWsssConfig::parse_file(&file.to_string_lossy()).ok(),)
}

#[cfg(test)]
mod tests {

  extern crate tempfile;

  use std::io::SeekFrom;

  use self::tempfile::*;
  use super::*;

  #[test]
  fn toml_wsss_config_default() {
    let cfg = TomlWsssConfig::default();
    assert_eq!(cfg.http_port, None, "Http port should be None");
    assert_eq!(cfg.ws_port, None, "WS port should be None");
    assert_eq!(cfg.bind_address, None, "bind address should be None");
  }

  #[test]
  fn wsss_config_default() {
    let cfg = WsssConfig::default();
    let def_bind_addr = Ipv4Addr::from_str(DEFAULT_BIND_ADDR).unwrap();
    assert_eq!(
      cfg.http_port,
      DEFAULT_HTTP_PORT,
      "Http port should be '{}'",
      DEFAULT_HTTP_PORT
    );
    assert_eq!(
      cfg.ws_port,
      DEFAULT_WS_PORT,
      "WS port should be '{}'",
      DEFAULT_WS_PORT
    );
    assert_eq!(
      cfg.bind_address,
      def_bind_addr,
      "bind address should be '{}'",
      DEFAULT_BIND_ADDR
    );
  }

  #[test]
  fn test_file_config() {
    let mut tmp_cfg_file = tempfile().expect("Creating temp file failed");
    let cfg = WsssConfig {
      http_port: 12345,
      ws_port: 12346,
      bind_address: Ipv4Addr::from_str("10.1.100.10").expect("Create config obj failed"),
    };
    let cfg_str = toml::to_string(&cfg).expect("Serializing to toml failed");
    tmp_cfg_file.write_all(cfg_str.as_bytes()).unwrap();
    tmp_cfg_file.flush().unwrap();
    tmp_cfg_file.seek(SeekFrom::Start(0)).unwrap();
    let mut contents = String::new();
    tmp_cfg_file.read_to_string(&mut contents).unwrap();
    let toml_cfg: TomlWsssConfig = toml::from_str(&contents).unwrap();
    let read_cfg = WsssConfig::from(toml_cfg);
    assert_eq!(
      cfg,
      read_cfg,
      "Configuration '{}' should equal '{}'",
      cfg_str,
      contents
    );
  }


}