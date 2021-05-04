#![allow(unused)]
use knight_lang::*;


fn main (){
	let mut env = Environment::default();
	println!("{:#?}", functions::get(&[
		Text::new("ABCDEFGHI").unwrap().into(),
		Number::new(0).unwrap().into(),
		Number::new(1).unwrap().into(),
		// Text::new("ba").unwrap().into()
	], &mut env));
}