#![allow(unused)]

use std::path::Path;

use knights_bytecode::env::Environment;
use knights_bytecode::parser::*;
use knights_bytecode::program::*;
use knights_bytecode::strings::StringSlice;
use knights_bytecode::value::*;
use knights_bytecode::vm::*;

fn main() {
	let mut env = Environment::default();
	let mut parser = Parser::new(
		&mut env,
		Some(Path::new("main")),
		r#"

; = (numer) 1
; = divide BLOCK
	/ numer denom
; = call_divide BLOCK
	CALL divide
; = denom 0
: OUTPUT CALL call_divide



# Fizzbuzz in Knight

# Initialize variables.
; = maximum 100
; = i 0

# Repeat the body while `i < maximum`.
: WHILE < i maximum
	# Increment `i`
	; = i + i 1

	# Use the fact that `IF` is an expression, not a statement like in some
	# languages (eg python, javascript, etc).
	: OUTPUT
		: IF ! % i 15 "FizzBuzz"
		: IF ! % i 5  "Fizz"
		: IF ! % i 3  "Buzz" i
"#,
	)
	.unwrap();

	let program = parser.parse_program().map_err(|err| panic!("{}", err)).unwrap();
	match Vm::new(&program, &mut env).run() {
		Ok(_) => {}
		Err(err) => eprintln!("error: {err}"),
	}
}

#[cfg(any())]
fn main() {
	let mut env = Environment::default();
	let mut builder = Program::builder(env.opts());
	// 		/*
	// 		; = n 10 var=1
	// 		; = i 0 var=0
	// 		; WHILE n
	// 			; = i + i n
	// 			: = n - n 1
	// 		OUTPUT i
	// 					*/
	let program = unsafe {
		// = n 10
		builder.push_constant(Integer::new_unvalidated(10).into());
		builder.set_variable_pop(StringSlice::new_unvalidated("n"));

		// = i 10
		builder.push_constant(Integer::new_unvalidated(0).into());
		builder.set_variable_pop(StringSlice::new_unvalidated("i"));

		// WHILE n
		let start = builder.jump_index();
		builder.get_variable(StringSlice::new_unvalidated("n"));
		let jump_to_end = builder.defer_jump(JumpWhen::False);

		// = i + i n
		builder.get_variable(StringSlice::new_unvalidated("n"));
		builder.get_variable(StringSlice::new_unvalidated("i"));
		builder.add();
		builder.set_variable_pop(StringSlice::new_unvalidated("i"));

		// = n - n 1
		builder.push_constant(Integer::new_unvalidated(1).into());
		builder.get_variable(StringSlice::new_unvalidated("n"));
		builder.sub();
		builder.set_variable_pop(StringSlice::new_unvalidated("n"));

		// go back to top of while
		builder.jump_to(JumpWhen::Always, start);
		jump_to_end.jump_to_current(&mut builder);

		// OUTPUT i
		builder.get_variable(StringSlice::new_unvalidated("i"));
		builder.output();
		builder.build()
	};

	let mut vm = Vm::new(&program, &mut env);
	&vm.run().unwrap();
}

// // Arity 1: :, BLOCK, CALL, QUIT, DUMP, OUTPUT, LENGTH, !, ~, ASCII, ,, [, ]
// // Arity 2: +, -, *, /, %, ^, <, >, ?, &, |, ;, =, WHILE
// // Arity 3: IF, GET
// // Arity 4: SET
// // }
// #[rustfmt::skip]
// pub fn foo() -> Program {
// 	fn op(opcode: Opcode, offset: u32) -> u32 {
// 		((opcode as u8 as u32)) | (offset << 0o10)
// 	}
// 	use Opcode::*;
// 	Program {
// 		code: vec![
// 			op(PushConstant, 2),
// 			op(SetVarPop, 0),
// 			op(PushConstant, 0),
// 			op(SetVarPop, 1),
// 			// WHILE
// 			op(GetVar, 0),
// 			op(JumpIfFalse, 15),
// 			op(GetVar, 0),
// 			op(GetVar, 1),
// 			op(Add, 0xff),
// 			op(SetVarPop, 1),
// 			op(PushConstant, 1),
// 			op(GetVar, 0),
// 			op(Sub, 0xff),
// 			op(SetVarPop, 0),
// 			op(Jump, 4),
// 			op(GetVar, 1),
// 			op(Output, 0xff),
// 			op(Return, 0xff) ,
// 		]
// 		.into(),
// 		constants: vec![
// 			Value::Integer(Integer::new_unvalidated(0)),
// 			Value::Integer(Integer::new_unvalidated(1)),
// 			Value::Integer(Integer::new_unvalidated(10)),
// 		]
// 		.into(),
// 		num_variables: 2,
// 	}
// }
