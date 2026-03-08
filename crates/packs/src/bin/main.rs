use packs::cli;

fn main() {
	let root = cli::Cli::new();
	root.parse();
}
