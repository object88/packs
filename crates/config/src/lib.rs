use std::io::{BufReader, Read};

use packs_logging::ConfigLevelFilter;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
	_log_level: ConfigLevelFilter,
}

impl Config {
	pub fn from_stream<R: Read>(stream: BufReader<R>) -> Config {
		match serde_json::from_reader(stream) {
			Ok(cfg) => cfg,
			Err(_e) => {},
		}
		Config::default()
	}
}
