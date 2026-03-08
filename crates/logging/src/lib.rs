use std::fmt::{self, Display};

use serde::{
	Deserialize,
	de::{self, Visitor},
};
use tracing::level_filters::LevelFilter;

const OFF: &str = "off";
const ERROR: &str = "error";
const WARN: &str = "warn";
const INFO: &str = "info";
const DEBUG: &str = "debug";
const TRACE: &str = "trace";

pub const DEFAULT: ConfigLevelFilter = ConfigLevelFilter::Warn;

/// ConfigLevelFilter is a wrapper around the `tracing` crate's `LevelFilter`
/// values.  It adds the ability to use with serde for deserialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigLevelFilter {
	Off,
	Error,
	Warn,
	Info,
	Debug,
	Trace,
}

impl Default for ConfigLevelFilter {
	fn default() -> Self {
		DEFAULT
	}
}

impl Display for ConfigLevelFilter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.to_str())
	}
}

struct ConfigLevelFilterVisitor;

impl<'de> Visitor<'de> for ConfigLevelFilterVisitor {
	type Value = ConfigLevelFilter;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_fmt(format_args!(
			"one of {}",
			ConfigLevelFilter::str_variants().join(",")
		))
	}

	fn visit_str<E: de::Error>(self, value: &str) -> Result<ConfigLevelFilter, E> {
		match value {
			OFF => Ok(ConfigLevelFilter::Off),
			ERROR => Ok(ConfigLevelFilter::Error),
			WARN => Ok(ConfigLevelFilter::Warn),
			INFO => Ok(ConfigLevelFilter::Info),
			DEBUG => Ok(ConfigLevelFilter::Debug),
			TRACE => Ok(ConfigLevelFilter::Trace),
			other => Err(E::unknown_variant(other, ConfigLevelFilter::str_variants())),
		}
	}
}

/// Using serde to deserialize the ConfigLevelFilter
/// ```
/// #[derive(serde::Deserialize)]
/// struct Foo {
///	  a: logging::ConfigLevelFilter,
/// }
/// # fn main() {
/// let raw = r#"{"a":"off"}"#;
///	let f: Foo = serde_json::from_str(raw).unwrap();
/// assert!(f.a == logging::ConfigLevelFilter::Off)
/// # }
/// ```
///
/// This is a runtime error
/// ```should_panic
/// #[derive(serde::Deserialize)]
/// struct Foo {
///	  a: logging::ConfigLevelFilter,
/// }
/// # fn main() {
/// let raw = r#"{}"#;
///	let f: Foo = serde_json::from_str(raw).unwrap();
/// # }
/// ```
impl<'de> Deserialize<'de> for ConfigLevelFilter {
	fn deserialize<D>(d: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		d.deserialize_str(ConfigLevelFilterVisitor)
	}
}

impl From<ConfigLevelFilter> for LevelFilter {
	fn from(val: ConfigLevelFilter) -> Self {
		match val {
			ConfigLevelFilter::Off => LevelFilter::OFF,
			ConfigLevelFilter::Error => LevelFilter::ERROR,
			ConfigLevelFilter::Warn => LevelFilter::WARN,
			ConfigLevelFilter::Info => LevelFilter::INFO,
			ConfigLevelFilter::Debug => LevelFilter::DEBUG,
			ConfigLevelFilter::Trace => LevelFilter::TRACE,
		}
	}
}

impl ConfigLevelFilter {
	pub const fn value_variants<'a>() -> &'a [Self] {
		&[
			ConfigLevelFilter::Off,
			ConfigLevelFilter::Error,
			ConfigLevelFilter::Warn,
			ConfigLevelFilter::Info,
			ConfigLevelFilter::Debug,
			ConfigLevelFilter::Trace,
		]
	}

	const fn str_variants() -> &'static [&'static str] {
		&[OFF, ERROR, WARN, INFO, DEBUG, TRACE]
	}

	pub const fn to_str(self) -> &'static str {
		match self {
			ConfigLevelFilter::Off => OFF,
			ConfigLevelFilter::Error => ERROR,
			ConfigLevelFilter::Warn => WARN,
			ConfigLevelFilter::Info => INFO,
			ConfigLevelFilter::Debug => DEBUG,
			ConfigLevelFilter::Trace => TRACE,
		}
	}
}
