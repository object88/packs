use packs_logging::ConfigLevelFilter;
use serde::Deserialize;
use serde_json::{Result, from_str};

#[derive(Deserialize)]
struct Foo {
	a: ConfigLevelFilter,
}

#[derive(Deserialize)]
struct Bar {
	a: Option<ConfigLevelFilter>,
}

#[test]
fn test_deserialize() {
	let raw = r#"{"a":"off"}"#;
	let f: Foo = from_str(raw).unwrap();
	assert!(f.a == ConfigLevelFilter::Off, "");
}

#[test]
fn test_deserialize_bad_value() {
	let raw = r#"{"a":"quux"}"#;
	let x: Result<Foo> = from_str(raw);
	assert!(x.is_err(), "");
}

#[test]
fn test_deserialize_missing_value() {
	let raw = r#"{}"#;
	let x: Result<Foo> = from_str(raw);
	assert!(x.is_err(), "");
}

#[test]
fn test_deserialize_missing_optional_value() {
	let raw = r#"{}"#;
	let f: Bar = from_str(raw).unwrap();
	assert!(f.a == None, "expected None, received {}", f.a.unwrap());
}
