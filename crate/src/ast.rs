use crate::{Environment, Function, RefCount, Value};

/// [`Ast`]s represent functions and their arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct Ast<'f>(RefCount<(&'f Function, Box<[Value<'f>]>)>);

impl<'f> Ast<'f> {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `func.arity`.
	pub fn new(func: &'f Function, args: Box<[Value<'f>]>) -> Self {
		assert_eq!(args.len(), func.arity);

		Self((func, args).into())
	}

	/// Gets the function associated with the ast.
	pub fn function(&self) -> &'f Function {
		(self.0).0
	}

	/// Executes the function associated with `self`.
	pub fn run(&self, env: &mut Environment<'f>) -> crate::Result<Value<'f>> {
		(self.function().func)(&(self.0).1, env)
	}
}
