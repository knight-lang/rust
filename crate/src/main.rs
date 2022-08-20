use knightrs::*;

fn main() {
	match Environment::default()
		.play(std::env::args().skip(2).next().unwrap().as_str().try_into().unwrap())
	{
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
