use knightrs::*;

fn main() {
	let mut env = Environment::default();
	let arg = SharedText::try_from(std::env::args().nth(2).expect("no arg")).unwrap();

	let arg = if std::env::args().nth(1).unwrap() == "-e" {
		arg
	} else {
		std::fs::read_to_string(&**arg).unwrap().as_str().try_into().unwrap()
	};

	match env.play(&arg) {
		Err(Error::Quit(code)) => std::process::exit(code),
		Err(err) => {
			eprintln!("error: {err}");
			std::process::exit(1);
		}
		_ => {}
	}
	/*
				r##"
	; = å = j 0
	; WHILE < å 100
		; = j + j å
		: = å + å 1
	; O j
	; = a 3
	#: O + a a
	O + 3 "   -4a"

	"##
				.try_into()
				.unwrap(),*/
}
