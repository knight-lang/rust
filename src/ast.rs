use crate::parse::{self, Parsable, Parser};
use crate::value::{integer::IntType, Runnable, Value};
use crate::{Environment, Function, RefCount, Result};

/// [`Ast`]s represent functions and their arguments.
#[derive(Debug)]
pub struct Ast<'e, I>(RefCount<Inner<'e, I>>);
impl<I> Clone for Ast<'_, I> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

#[derive(Debug)]
struct Inner<'e, I> {
	function: Function<'e, I>,
	args: Box<[Value<'e, I>]>,
}

impl<I> Eq for Ast<'_, I> {}
impl<I> PartialEq for Ast<'_, I> {
	/// Two `Ast`s are equal only if they point to the exact same data.
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl<I> crate::value::NamedType for Ast<'_, I> {
	const TYPENAME: &'static str = "Ast";
}

impl<'e, I: IntType> Ast<'e, I> {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `function.arity`.
	#[must_use]
	#[inline]
	pub fn new(function: Function<'e, I>, args: Box<[Value<'e, I>]>) -> Self {
		assert_eq!(args.len(), function.arity());

		Self(Inner { function, args }.into())
	}

	/// Gets the function associated with the ast.
	#[must_use]
	#[inline]
	pub fn function(&self) -> &Function<'e, I> {
		&self.0.function
	}

	/// Gets the args associated with the ast.
	#[must_use]
	#[inline]
	pub fn args(&self) -> &[Value<'e, I>] {
		&self.0.args
	}
}

impl<'e, I: IntType> Runnable<'e, I> for Ast<'e, I> {
	#[inline]
	fn run(&self, env: &mut Environment<'e, I>) -> Result<Value<'e, I>> {
		self.function().run(self.args(), env)
	}
}

impl<'e, I: IntType> Parsable<'e, I> for Ast<'e, I> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, 'e, I>) -> parse::Result<Option<Self>> {
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
						ErrorKind::MissingArgument { name: function.full_name().to_owned(), index }
							.error(start_line),
					)
				}
				Err(err) => return Err(err),
			}
		}

		Ok(Some(Self::new(function, args.into())))
	}
}
