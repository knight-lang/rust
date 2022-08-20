use knightrs::*;

fn main() {
	Environment::default()
		.play(
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
			.unwrap(),
		)
		.unwrap();
}
