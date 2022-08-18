use crate::{Environment, Function, Result, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Ast(Box<(&'static Function, Vec<Value>)>);

impl Ast {
	/// Creates a new `Ast` from the given arguments.
	///
	/// This will panic if `args.len()` isnt equal to `func.arity.`
	pub fn new(func: &'static Function, args: Vec<Value>) -> Self {
		assert_eq!(args.len(), func.arity);

		Self(Box::new((func, args)))
	}

	/// Gets the function associated with the ast.
	pub fn function(&self) -> &'static Function {
		(self.0).0
	}

	/// Executes the function associated with `self`.
	pub fn run(&self, env: &mut Environment<'_>) -> Result<Value> {
		(self.function().func)(&(self.0).1, env)
	}
}
