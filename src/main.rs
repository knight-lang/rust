fn main() {
	let arg = std::env::args().nth(2).expect("no arg");

	let arg = if std::env::args().nth(1).unwrap() == "-e" {
		arg
	} else {
		std::fs::read_to_string(&*arg).unwrap()
	};

	match knightrs::play("unicode", "i64", "wrapping", &arg, &Default::default()) {
		Err(knightrs::Error::Quit(code)) => std::process::exit(code),
		Err(err) => {
			eprintln!("error: {err}");
			std::process::exit(1);
		}
		_ => {}
	}
}
