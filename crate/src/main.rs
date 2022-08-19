use crate::parser::Parser;
use knightrs::*;

fn main() {
	let parser = Parser::new(
		r##"
; = i = j 0
; WHILE < i 100
	; = j + j i
	: = i + i 1
: O j
; = a 3
#: O + a a
1

"##,
	)
	.unwrap();
	let mut env = Environment::default();
	parser.parse_program(&mut env).unwrap().run(&mut env).unwrap();
}
