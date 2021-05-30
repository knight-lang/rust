#![allow(unused)]
use knight::*;
use knight::value::ValueKind;


static FUNCTION: Function = Function {
	name: 'D',
	arity: 1,
	func: |args| { println!("{:?}", args[0]); Ok(args[0].clone()) }
};

fn main() {
	let ast = Ast::new(&FUNCTION, vec![Value::from(Text::new("A".into()).unwrap())].into());
	dbg!(ast);

// let _ = dbg!(ast.run());
// dbg!(Value::from(true));
// dbg!(Value::from(false));
// dbg!(Value::from(Null));
// dbg!(Value::from(Number::new(123).unwrap()));
// dbg!(Value::from(Text::new("123".into()).unwrap()));
// dbg!(Value::from(ast));
}
