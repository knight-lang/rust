use crate::parse::{self, Parsable, Parser};
use crate::value::{integer::IntType, text::Encoding, Runnable, Value};
use crate::{Environment, Function, RefCount, Result};
use std::fmt::Debug;

/// [`Ast`]s represent functions and their arguments.
#[derive_where(Debug; I: Debug)]
#[derive_where(Clone)]
pub struct Ast<'e, I, E>(RefCount<Inner<'e, I, E>>);

#[derive_where(Debug; I: Debug)]
struct Inner<'e, I, E> {
	function: Function<'e, I, E>,
	args: Box<[Value<'e, I, E>]>,
}

impl<I, E> Eq for Ast<'_, I, E> {}
impl<I, E> PartialEq for Ast<'_, I, E> {
	/// Two `Ast`s are equal only if they point to the exact same data.
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl<I, E> crate::value::NamedType for Ast<'_, I, E> {
	const TYPENAME: &'static str = "Ast";
}

impl<'e, I, E> Ast<'e, I, E> {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `function.arity`.
	#[must_use]
	pub fn new(function: Function<'e, I, E>, args: Box<[Value<'e, I, E>]>) -> Self {
		assert_eq!(args.len(), function.arity());

		Self(Inner { function, args }.into())
	}

	/// Gets the function associated with the ast.
	#[must_use]
	pub fn function(&self) -> &Function<'e, I, E> {
		&self.0.function
	}

	/// Gets the args associated with the ast.
	#[must_use]
	pub fn args(&self) -> &[Value<'e, I, E>] {
		&self.0.args
	}
}

impl<'e, I: IntType, E: Encoding> Runnable<'e, I, E> for Ast<'e, I, E> {
	fn run(&self, env: &mut Environment<'e, I, E>) -> Result<Value<'e, I, E>> {
		self.function().run(self.args(), env)
	}
}

impl<'e, I: IntType, E: Encoding> Parsable<'e, I, E> for Ast<'e, I, E> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, 'e, I, E>) -> parse::Result<Option<Self>> {
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
