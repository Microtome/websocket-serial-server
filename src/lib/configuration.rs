//! Loads and centralizes configuration from
//! config files, env, and command line
//!
//! For information on command line switches, config files,
//! or env names, check the documentation for [WsspsConfig](struct.WsspsConfig.html)
//!
//! TODO: Tls Support

use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use figment::providers::*;
use figment::Figment;
use serde_with::skip_serializing_none;
use structopt::StructOpt;

/// Default HTTP port to bind to if none given
pub const DEFAULT_PORT: u16 = 10080;
/// Default ip address to bind
pub const DEFAULT_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

/// Suported config file extensions
pub const SUPPORTED_EXTENSIONS: &[&str] = &["toml", "json", "yaml"];

/// Config file base name
pub const CONFIG_FILE_NAME_PREFIX: &str = "wssps_conf";

/// Configuration settings.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, StructOpt)]
#[structopt(name = "wssps", about = "WebSocket Serial Port Server")]
pub struct WsspsConfigCli {
    /// The port to listen on.
    #[structopt(short, long, help = "port to bind to")]
    pub port: Option<u16>,

    /// Address to bind to.
    #[structopt(short, long, help = "address to bind")]
    pub address: Option<IpAddr>,

    /// Configuration file to use
    #[structopt(short = "f", long = "config-file", help = "address to bind")]
    pub configuration_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsspsConfig {
    /// The port to listen on.
    pub port: u16,

    /// Address to bind to.
    pub address: IpAddr,
}

impl Default for WsspsConfig {
  fn default() -> Self {
      Self {
          port: DEFAULT_PORT,
          address: DEFAULT_ADDRESS
      }
  }
}

impl WsspsConfig {
    pub fn get() -> Self {
        let cli_args: WsspsConfigCli = WsspsConfigCli::from_args();

        // Pull from CLI, fill holes with env
        let figment_cli = Figment::from(Serialized::defaults(WsspsConfig::default()))
            .merge(Env::prefixed("WSSPS_"))
            .merge(Serialized::defaults(cli_args));

        let env_args: WsspsConfigCli = figment_cli
            .extract()
            .expect("Failed to load config from environment args"); // Shouldn't happen

        // Should have all values filled 
        // let base_config: WsspsConfig = figment.extract().expect("Should not fail");

        let figment = match env_args.configuration_file {
            Some(configuration_file) => Figment::from(figment_cli).join_file(configuration_file),
            None => figment_cli,
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
            .expect("Could not determine config file type")
            .to_string_lossy()
            .as_ref()
        {
            "yaml" | "yml" => self.join(Yaml::file(file_path)),
            "json" => self.join(Yaml::file(file_path)),
            "toml" => self.join(Yaml::file(file_path)),
            _ => panic!("Format not supported"),
        }
    }
}
