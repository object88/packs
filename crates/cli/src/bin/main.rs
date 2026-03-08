use cli::cli;

fn main() {
	let root = cli::Cli::new();
	root.parse();
}
