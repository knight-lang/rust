use knights_bytecode::env::Environment;
use knights_bytecode::vm::*;

fn main() {
	let mut env = Environment::default();
	let foo = foo();
	let mut vm = Vm::new(&foo, &mut env);
	vm.run().unwrap();
}
