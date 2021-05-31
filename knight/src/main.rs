#![allow(unused)]
use knight_lang::*;
use knight_lang::value::ValueKind;

fn main(){}

// // static FUNCTION: Function = Function {
// // 	name: 'D',
// // 	arity: 1,
// // 	func: |args, _| { println!("{:?}", args[0]); Ok(args[0].clone()) }
// // };

// fn main() {
// 	let mut env = Environment::default();
// 	let ast = Ast::new(&FUNCTION, vec![Value::from(Text::new("A".into()).unwrap())].into());

// 	let foo = env.fetch_var("foo".into());

// 	dbg!(foo);
// 	foo.set(Value::from(Number::new(123).unwrap()));
// 	dbg!(foo);
// 	dbg!(env.fetch_var("foo".into()));

// 	// let _ = dbg!(ast.run(&mut env));
// // dbg!(Value::from(true));
// // dbg!(Value::from(false));
// // dbg!(Value::from(Null));
// // dbg!(Value::from(Number::new(123).unwrap()));
// // dbg!(Value::from(Text::new("123".into()).unwrap()));
// // dbg!(Value::from(ast));
// }
