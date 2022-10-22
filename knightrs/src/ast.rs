use crate::parse::{self, Parsable, Parser};
use crate::value::{integer::IntType, Runnable, Value};
use crate::{Environment, Function, RefCount, Result};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// [`Ast`]s represent functions and their arguments.
#[derive_where(Debug; I: Debug)]
#[derive_where(Clone)]
pub struct Ast<I>(RefCount<Inner<I>>);

#[derive_where(Debug; I: Debug)]
struct Inner<I> {
	function: Function<I>,
	args: Box<[Value<I>]>,
}

impl<I> Eq for Ast<I> {}
impl<I> PartialEq for Ast<I> {
	/// Two `Ast`s are equal only if they point to the exact same data.
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl<I: Hash> Hash for Ast<I> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(RefCount::as_ptr(&self.0) as usize).hash(state);
	}
}

impl<I> crate::value::NamedType for Ast<I> {
	const TYPENAME: &'static str = "Ast";
}

impl<I> Ast<I> {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `function.arity`.
	#[must_use]
	pub fn new(function: Function<I>, args: Box<[Value<I>]>) -> Self {
		assert_eq!(args.len(), function.arity());

		Self(Inner { function, args }.into())
	}

	/// Gets the function associated with the ast.
	#[must_use]
	pub fn function(&self) -> &Function<I> {
		&self.0.function
	}

	/// Gets the args associated with the ast.
	#[must_use]
	pub fn args(&self) -> &[Value<I>] {
		&self.0.args
	}
}

impl<I: IntType> Runnable<I> for Ast<I> {
	fn run(&self, env: &mut Environment<'_, I>) -> Result<Value<I>> {
		self.function().run(self.args(), env)
	}
}

impl<I: IntType> Parsable<I> for Ast<I> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, I>) -> parse::Result<Option<Self>> {
		use parse::{Error, ErrorKind};

		let Some(function) = Function::parse(parser)? else {
			return Ok(None);
		};

		// `MissingArgument` errors have their `line` field set to the beginning of the function
		// parsing.
		let start_line = parser.line();
		let mut args = Vec::with_capacity(function.arity());

		for index in 0..function.arity() {
			match parser.parse_expression() {
				Ok(arg) => args.push(arg),
				Err(Error { kind: ErrorKind::EmptySource, .. }) => {
					return Err(
						ErrorKind::MissingArgument { name: function.full_name().to_string(), index }
							.error(start_line),
					)
				}
				Err(err) => return Err(err),
			}
		}

		Ok(Some(Self::new(function, args.into())))
	}
}
