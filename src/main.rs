use knightrs::*;

fn main() {
	let mut env = Environment::<'_, i32>::default();
	let arg = Text::new(std::env::args().nth(2).expect("no arg"), env.flags()).unwrap();

	let arg = if std::env::args().nth(1).unwrap() == "-e" {
		arg
	} else {
		Text::new(std::fs::read_to_string(&**arg).unwrap(), env.flags()).unwrap()
	};

	match env.play(&arg) {
		Err(Error::Quit(code)) => std::process::exit(code),
		Err(err) => {
			eprintln!("error: {err}");
			std::process::exit(1);
		}
		_ => {}
	}
	drop(env);
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
