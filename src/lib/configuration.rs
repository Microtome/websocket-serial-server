//! Loads and centralizes configuration from
//! config files, env, and command line
//!
//! For information on command line switches, config files,
//! or env names, check the documentation for [WsspsConfig](struct.WsspsConfig.html)
//!
//! TODO: Tls Support

use std::{net::Ipv4Addr, path::PathBuf};

use structopt::StructOpt;

use figment::providers::*;
use figment::Figment;

/// Default HTTP port to bind to if none given
pub const DEFAULT_PORT: u32 = 10080;
/// Default ip address to bind
pub const DEFAULT_ADDRESS: &str = "127.0.0.1";

/// Suported config file extensions
pub const SUPPORTED_EXTENSIONS: &[&str] = &["toml", "json", "yaml"];

/// Config file base name
pub const CONFIG_FILE_NAME_PREFIX: &str = "wssps_conf";

/// Configuration settings.
#[derive(Clone, Debug, Serialize, Deserialize, StructOpt)]
#[structopt(name = "wssps", about = "WebSocket Serial Port Server")]
pub struct WsspsConfigCli {
  /// The port to listen on.
  #[structopt(short, long, help = "port to bind to")]
  pub port: Option<u32>,

  /// Address to bind to.
  #[structopt(short, long, help = "address to bind")]
  pub address: Option<Ipv4Addr>,

  /// Configuration file to use
  #[structopt(short = "f", long = "config-file", help = "address to bind")]
  pub configuration_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsspsConfig {
  /// The port to listen on.
  pub port: u32,

  /// Address to bind to.
  pub address: Ipv4Addr,
}
impl WsspsConfig {
  fn get() -> Self {
    let cli_args: WsspsConfigCli = WsspsConfigCli::from_args();

    // Pull from CLI, fill holes with env
    let figment = Figment::from(Serialized::defaults(cli_args)).join(Env::prefixed("WSSPS_"));

    let env_args: WsspsConfigCli = figment
      .extract()
      .expect("Failed to load config from environment args"); // Shouldn't happen

    let figment = match env_args.configuration_file {
      Some(configuration_file) => figment.join_file(configuration_file),
      None => figment,
    };

    figment
      .extract::<WsspsConfig>()
      .expect("Failed to parse configuration")
  }
}

// TODO Makes this a true provider instead of panicing.
pub trait FigmentExt {
  fn join_file(self, file_path: PathBuf) -> Figment;
}

impl FigmentExt for Figment {
  fn join_file(self, file_path: PathBuf) -> Figment {
    match file_path
      .extension()
      .expect("Could determine config file type")
      .to_string_lossy()
      .as_ref()
    {
      "yaml" | "yml" => self.join(Yaml::file(file_path)),
      "json" => self.join(Yaml::file(file_path)),
      "toml" => self.join(Yaml::file(file_path)),
      _ =>  panic!("Format not supported")
    }
  }
}
