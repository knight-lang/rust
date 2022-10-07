use crate::value::{Runnable, Value};
use crate::{Environment, Function, RefCount, Result};

/// [`Ast`]s represent functions and their arguments.
#[derive(Debug, Clone)]
pub struct Ast<'e>(RefCount<Inner<'e>>);

#[derive(Debug)]
struct Inner<'e> {
	function: &'e Function,
	args: Box<[Value<'e>]>,
}

impl Eq for Ast<'_> {}
impl PartialEq for Ast<'_> {
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl crate::value::NamedType for Ast<'_> {
	const TYPENAME: &'static str = "Ast";
}

impl<'e> Ast<'e> {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `function.arity`.
	#[must_use]
	#[inline]
	pub fn new(function: &'e Function, args: Box<[Value<'e>]>) -> Self {
		assert_eq!(args.len(), function.arity);

		Self(Inner { function, args }.into())
	}

	/// Gets the function associated with the ast.
	#[must_use]
	#[inline]
	pub fn function(&self) -> &'e Function {
		self.0.function
	}

	/// Gets the args associated with the ast.
	#[must_use]
	#[inline]
	pub fn args(&self) -> &[Value<'e>] {
		&self.0.args
	}
}

impl<'e> Runnable<'e> for Ast<'e> {
	#[inline]
	fn run(&self, env: &mut Environment<'e>) -> Result<Value<'e>> {
		(self.function().func)(self.args(), env)
	}
}
