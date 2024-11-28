use knights_bytecode::env::Env;
use knights_bytecode::vm::*;

fn main() {
	let env = Env::default();
	let foo = foo();
	let mut vm = Vm::new(&foo, &env);
	vm.run().unwrap();
}
