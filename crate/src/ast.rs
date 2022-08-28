use crate::{Environment, Function, RefCount, Value};

/// [`Ast`]s represent functions and their arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct Ast(RefCount<(&'static Function, Box<[Value]>)>);

impl Ast {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `func.arity`.
	pub fn new(func: &'static Function, args: Box<[Value]>) -> Self {
		assert_eq!(args.len(), func.arity);

		Self((func, args).into())
	}

	/// Gets the function associated with the ast.
	pub fn function(&self) -> &'static Function {
		(self.0).0
	}

	/// Executes the function associated with `self`.
	pub fn run(&self, env: &mut Environment) -> crate::Result<Value> {
		(self.function().func)(&(self.0).1, env)
	}
}
