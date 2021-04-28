#![allow(unused)]
use knight_lang::*;


fn main() {
	let mut builder = Text::builder(3);
	builder.write(b"abcd");
	builder.write(b"1");

	println!("{:?}", builder.build());
	// let text = Text::from("foo");

	// println!("{:?}", text);
}