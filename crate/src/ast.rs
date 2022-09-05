use crate::{Environment, Function, RefCount, Value};

/// [`Ast`]s represent functions and their arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct Ast<'e>(RefCount<(&'e Function, Box<[Value<'e>]>)>);

impl<'e> Ast<'e> {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `func.arity`.
	#[must_use]
	pub fn new(func: &'e Function, args: Box<[Value<'e>]>) -> Self {
		assert_eq!(args.len(), func.arity);

		Self((func, args).into())
	}

	/// Gets the function associated with the ast.
	#[must_use]
	pub fn function(&self) -> &'e Function {
		(self.0).0
	}

	/// Gets the args associated with the ast.
	#[must_use]
	pub fn args(&self) -> &[Value<'e>] {
		&(self.0).1
	}

	/// Executes the function associated with `self`.
	pub fn run(&self, env: &mut Environment<'e>) -> crate::Result<Value<'e>> {
		(self.function().func)(self.args(), env)
	}
}
