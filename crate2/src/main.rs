#![allow(unused)]
use knight_lang::*;


fn main() {
	let mut builder = Text::builder(3);
	builder.concat("abcd");
	builder.concat("1");

	println!("{:?}", builder.build());
	// let text = Text::from("foo");

	// println!("{:?}", text);
}