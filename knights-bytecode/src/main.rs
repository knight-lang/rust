use knights_bytecode::env::Environment;
use knights_bytecode::vm::*;

fn main() {
	let mut env = Environment::default();
	// let foo = foo();
	// let mut vm = Vm::new(&foo, &mut env);
	// vm.run().unwrap();
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
// 		/*
// 		; = n 10 var=1
// 		; = i 0 var=0
// 		; WHILE n
// 			; = i + i n
// 			: = n - n 1
// 		OUTPUT i
// 					*/
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
