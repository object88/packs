use std::{
	env,
	fmt::Display,
	fs::File,
	io::{BufReader, ErrorKind},
	path::PathBuf,
};

use anyhow::Result;
use clap::{
	Arg, ArgMatches, Command,
	builder::{EnumValueParser, PathBufValueParser},
	parser::ValueSource,
};
use config::Config;
use packs::devices::list;
use thiserror::Error;
use tracing::{trace, trace_span};

use crate::logging::{self, ArgLevelFilter};

const CMD_LIST_DEVICES: &str = "list-devices";
const CMD_VERSION: &str = "version";

const FLAG_CONFIG_FILE: &str = "config-file";
const FLAG_LOG_LEVEL: &str = "log-level";
const FLAG_OUTPUT: &str = "output";

#[derive(Debug, Error)]
enum ConfigLoadError {
	#[error("")]
	InternalErrorMissingDefault,

	#[error("")]
	InternalErrorUnknownValueSource,

	#[error("")]
	InternalErrorNoValueSource,

	#[error("")]
	InternalErrorEnvUnsupported,
}

enum Cmd {
	ListDevices,
	Version,
}

impl Display for Cmd {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Cmd::ListDevices => write!(f, "{}", CMD_LIST_DEVICES),
			Cmd::Version => write!(f, "{}", CMD_VERSION),
		}
	}
}

impl Into<clap::builder::Str> for Cmd {
	fn into(self) -> clap::builder::Str {
		match self {
			Cmd::ListDevices => CMD_LIST_DEVICES.into(),
			Cmd::Version => CMD_VERSION.into(),
		}
	}
}

pub struct Cli {
	root: Command,
}

impl Cli {
	pub fn new() -> Cli {
		const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

		// Get the config directory
		let config_dir = match dirs::config_dir() {
			Some(config_dir) => config_dir.join(CRATE_NAME),
			None => PathBuf::from(""),
		};

		let root = Command::new(CRATE_NAME)
			.arg(
				Arg::new("config-file")
					.long(FLAG_CONFIG_FILE)
					.default_value(config_dir.into_os_string())
					.value_parser(PathBufValueParser::new()),
			)
			.arg(
				Arg::new("log-level")
					.long(FLAG_LOG_LEVEL)
					.default_value(logging::DEFAULT)
					.help("Set the log level")
					.long_help("Set the log level. 'trace' is the most verbose and 'off' the least verbose")
					.value_parser(EnumValueParser::<ArgLevelFilter>::new()),
			)
			.arg(Arg::new("output").long(FLAG_OUTPUT))
			.arg_required_else_help(true)
			.subcommand(Command::new(Cmd::ListDevices))
			.subcommand(Command::new(Cmd::Version));
		Cli { root }
	}

	pub fn parse(self) -> Result<()> {
		let matches = self.root.get_matches();

		// Configure logging first; let's figure out how to report back to the world.
		let log_level: &ArgLevelFilter = matches
			.get_one::<ArgLevelFilter>(FLAG_LOG_LEVEL)
			.unwrap_or_else(|| &logging::DEFAULT);
		logging::init(log_level);

		// Configuration file comes next
		let _cfg = Cli::load_config(&matches);

		match matches.subcommand() {
			Some((CMD_LIST_DEVICES, _sub)) => list(),
			Some((CMD_VERSION, _sub)) => Ok(()),
			_ => Ok(()),
		}
	}

	fn load_config(matches: &ArgMatches) -> Result<Config> {
		let span = trace_span!("config loading");
		let _enter = span.enter();

		match matches.value_source(FLAG_CONFIG_FILE) {
			Some(ValueSource::CommandLine) => {
				trace!("command line value source");
				// The config file has been specified on the command line.  If the value
				// is not present, this is a failure.
				match matches.get_one::<PathBuf>(FLAG_CONFIG_FILE) {
					Some(config_path) => {
						// Have some value
						trace!(path = ?config_path, "have config path");
						match File::open(config_path) {
							Ok(f) => return Ok(Config::from_stream(BufReader::new(f))),
							Err(e) => match e.kind() {
								ErrorKind::NotFound => {
									// There is no config file, and that is OK.
									return Ok(Config::default());
								},
								_ => {
									// This is an actual error, and should be surfaced.
									// TODO: customer this error?
									return Err(e.into());
								},
							},
						}
					},
					None => {
						// Not sure how we can get here.
						// TODO: attempt to reach here and figure out meaningful response
						return Ok(Config::default());
					},
				}
			},
			Some(ValueSource::DefaultValue) => {
				// Using the default value.  This file may or may not exist; if its not
				// present, then use the config's defaults.
				match matches.get_one::<PathBuf>(FLAG_CONFIG_FILE) {
					Some(config_path) => {
						// No overriding value was provided.  Its possible, however, that the
						// config directory could not be determined and the default was an
						// empty string, or that there is no file at the default location.
						// Handle all these cases.
						// The preferred `is_empty` method on Path is in the unstable build,
						// which we are not using.  Ref: https://github.com/rust-lang/rust/issues/148494
						// Will use a less ergonomic way.
						if config_path.components().next().is_none() {
							// There is no config path at all
							return Ok(Config::default());
						} else {
							// Have a path; if there is something there, read it.  If it cannot be
							// opened, its possible that the path does not exist, or that there was
							// an issue with locking or permissions.
							match File::open(config_path) {
								Ok(f) => return Ok(Config::from_stream(BufReader::new(f))),
								Err(e) => match e.kind() {
									ErrorKind::NotFound => {
										// There is no config file, and that is OK.
										return Ok(Config::default());
									},
									_ => {
										// This is an actual error, and should be surfaced.
										// TODO: customer this error?
										return Err(e.into());
									},
								},
							}
						}
					},
					None => {
						// Should not get to this point, since the `Arg` was set up with a
						// default value.  For completion sake, however, we want to cover this
						// potential case.
						return Err(ConfigLoadError::InternalErrorMissingDefault.into());
					},
				}
			},
			Some(ValueSource::EnvVariable) => {
				// Not expecting to use env vars at this point, but it may be an option
				// in the future.
				return Err(ConfigLoadError::InternalErrorEnvUnsupported.into());
			},
			Some(vs) => {
				// Need to include this because the `ValueSource` is marked as
				// non_exhaustive.  We should not get here.
				trace!("unknown value source: {:?}", vs);
				return Err(ConfigLoadError::InternalErrorUnknownValueSource.into());
			},
			None => {
				// Not clear when `ValueSource` would not be set.
				trace!("no value source");
				return Err(ConfigLoadError::InternalErrorNoValueSource.into());
			},
		}
	}
}
