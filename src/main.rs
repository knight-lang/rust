use knightrs::*;

fn main() {
	let mut stdout = Vec::new();
	let mut env = Environment::<Ascii, value::integer::Wrapping<i64>>::builder();
	env.stdout(&mut stdout);
	let mut env = env.build();
	let arg = Text::try_from(std::env::args().nth(2).expect("no arg")).unwrap();

	let arg = if std::env::args().nth(1).unwrap() == "-e" {
		arg
	} else {
		Text::try_from(std::fs::read_to_string(&**arg).unwrap()).unwrap()
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
	dbg!(stdout);
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
