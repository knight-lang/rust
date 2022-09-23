use crate::value::{Runnable, Value};
use crate::{Environment, Function, Result};
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

/// [`Ast`]s represent functions and their arguments.
pub struct Ast<'e, E, I>(Arc<(Function<'e, E, I>, Box<[Value<'e, E, I>]>)>);

impl<E, I> Debug for Ast<'_, E, I> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("Ast").field("fn", &self.function()).field("args", &self.args()).finish()
	}
}

impl<E, I> Clone for Ast<'_, E, I> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<E, I> PartialEq for Ast<'_, E, I> {
	fn eq(&self, rhs: &Self) -> bool {
		Arc::ptr_eq(&self.0, &rhs.0)
	}
}

impl<'e, E, I> Ast<'e, E, I> {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `func.arity`.
	#[must_use]
	pub fn new(func: Function<'e, E, I>, args: Box<[Value<'e, E, I>]>) -> Self {
		assert_eq!(args.len(), func.arity());

		Self((func, args).into())
	}

	/// Gets the function associated with the ast.
	#[must_use]
	pub fn function(&self) -> &Function<'e, E, I> {
		&(self.0).0
	}

	/// Gets the args associated with the ast.
	#[must_use]
	pub fn args(&self) -> &[Value<'e, E, I>] {
		&(self.0).1
	}
}

impl<'e, E, I> Runnable<'e, E, I> for Ast<'e, E, I> {
	fn run(&self, env: &mut Environment<'e, E, I>) -> Result<Value<'e, E, I>> {
		self.function().call(self.args(), env)
	}
}
