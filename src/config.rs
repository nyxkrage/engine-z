use std::net::{IpAddr, Ipv4Addr};
use std::ops::Range;
use std::path::PathBuf;

use configr::{Config, Configr};
use serde::{Deserialize, Serialize};

#[derive(Configr, Deserialize, Serialize, Debug, Clone)]
pub struct AppConfig {
	pub ip: IpAddr,
	pub port_range: Range<u16>,
	pub catch_all: CatchAllPort,

	pub web_root: PathBuf,
	pub default_project: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type", content = "value")]
pub enum CatchAllPort {
	Set(u16),
	First,
	Last,
	None,
}

impl Default for AppConfig {
	fn default() -> Self {
		Self {
			ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
			port_range: 42069..42100,
			catch_all: CatchAllPort::First,
			web_root: PathBuf::from("/var/www"),
			default_project: String::from("html"),
		}
	}
}
