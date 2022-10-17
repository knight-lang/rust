// use clap::{Parser, Subcommand};
// use std::path::PathBuf;

// #[derive(Parser)]
// #[command(author, version, about, long_about = None)]
// struct Cli {
// 	/// Optional name to operate on
// 	name: Option<String>,

// 	/// Sets a custom config file
// 	#[arg(short, long, value_name = "FILE")]
// 	config: Option<PathBuf>,

// 	/// Turn debugging information on
// 	#[arg(short, long, action = clap::ArgAction::Count)]
// 	debug: u8,
// 	// #[command(subcommand)]
// 	// command: Option<Commands>,
// }

// fn main() {
// 	let cli = Cli::parse();
// }

fn main() {
	let arg = std::env::args().nth(2).expect("no arg");

	let arg = if std::env::args().nth(1).unwrap() == "-e" {
		arg
	} else {
		std::fs::read_to_string(&*arg).unwrap()
	};

	// match knightrs::play("utf8", "i64", "wrapping", &arg, &Default::default()) {
	match knightrs::play("knight-encoding", "i32", "checked", &arg, &Default::default()) {
		Err(knightrs::Error::Quit(code)) => std::process::exit(code),
		Err(err) => {
			eprintln!("error: {err}");
			std::process::exit(1);
		}
		_ => {}
	}
}
