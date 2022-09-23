use crate::value::{Runnable, Value};
use crate::{Environment, Function, Result};
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

/// [`Ast`]s represent functions and their arguments.
pub struct Ast<'e, E>(Arc<(Function<'e, E>, Box<[Value<'e, E>]>)>);

impl<E> Debug for Ast<'_, E> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.debug_struct("Ast").field("fn", &self.function()).field("args", &self.args()).finish()
	}
}

impl<E> Clone for Ast<'_, E> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<E> PartialEq for Ast<'_, E> {
	fn eq(&self, rhs: &Self) -> bool {
		Arc::ptr_eq(&self.0, &rhs.0)
	}
}

impl<'e, E> Ast<'e, E> {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `func.arity`.
	#[must_use]
	pub fn new(func: Function<'e, E>, args: Box<[Value<'e, E>]>) -> Self {
		assert_eq!(args.len(), func.arity());

		Self((func, args).into())
	}

	/// Gets the function associated with the ast.
	#[must_use]
	pub fn function(&self) -> &Function<'e, E> {
		&(self.0).0
	}

	/// Gets the args associated with the ast.
	#[must_use]
	pub fn args(&self) -> &[Value<'e, E>] {
		&(self.0).1
	}
}

impl<'e, E> Runnable<'e, E> for Ast<'e, E> {
	fn run(&self, env: &mut Environment<'e, E>) -> Result<Value<'e, E>> {
		self.function().call(self.args(), env)
	}
}
