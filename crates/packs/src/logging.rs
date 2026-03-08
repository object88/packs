use std::{
	fmt::{Display, Formatter, Result},
	io::stderr,
	sync::LazyLock,
};

use clap::{
	ValueEnum,
	builder::{OsStr, PossibleValue},
};
use packs_logging::{ConfigLevelFilter, DEFAULT as CONFIGLEVELFILTER_DEFAULT};

/// ArgLevelFilter is a newtype for ConfigLevelFilter, so that `clap`'s
/// ValueEnum can be implemented.  Implementations are largely shallow wrappers
/// around ConfigLevelFilter.
#[derive(Clone)]
pub struct ArgLevelFilter(ConfigLevelFilter);

impl From<ArgLevelFilter> for ConfigLevelFilter {
	fn from(val: ArgLevelFilter) -> Self {
		val.0
	}
}

impl From<ArgLevelFilter> for OsStr {
	fn from(value: ArgLevelFilter) -> Self {
		value.0.to_str().into()
	}
}

pub const DEFAULT: ArgLevelFilter = ArgLevelFilter(CONFIGLEVELFILTER_DEFAULT);

impl Display for ArgLevelFilter {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		self.0.fmt(f)
	}
}

// ValueEnum is necessary for `clap`'s EnumValueParser
impl ValueEnum for ArgLevelFilter {
	fn value_variants<'a>() -> &'a [Self] {
		static VARIANTS: LazyLock<Vec<ArgLevelFilter>> = LazyLock::new(|| {
			ConfigLevelFilter::value_variants()
				.iter()
				.map(|x| ArgLevelFilter(*x))
				.collect()
		});
		&VARIANTS
	}

	fn to_possible_value(&self) -> Option<PossibleValue> {
		Some(PossibleValue::new(self.0.to_string().to_lowercase()))
	}
}

pub fn init(level: &ArgLevelFilter) {
	// Set up logging
	tracing_subscriber::fmt()
		.with_max_level(level.0)
		.with_writer(stderr)
		.init();
}
