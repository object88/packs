use anyhow::Result;
use cli::cli;

fn main() -> Result<()> {
	let root = cli::Cli::new();
	root.parse()
}
