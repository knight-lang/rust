use crate::parse::{self, Parsable, Parser};
use crate::value::{Runnable, Value};
use crate::{Environment, Function, RefCount, Result};

/// [`Ast`]s represent functions and their arguments.
#[derive(Debug, Clone)]
pub struct Ast<'e>(RefCount<Inner<'e>>);

#[derive(Debug)]
struct Inner<'e> {
	function: &'e Function<'e>,
	args: Box<[Value<'e>]>,
}

impl Eq for Ast<'_> {}
impl PartialEq for Ast<'_> {
	/// Two `Ast`s are equal only if they point to the exact same data.
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
	pub fn function(&self) -> &'e Function<'e> {
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

impl<'e> Parsable<'e> for Ast<'e> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, 'e>) -> parse::Result<Option<Self>> {
		use parse::{Error, ErrorKind};

		let Some(function) = <&Function>::parse(parser)? else {
			return Ok(None);
		};

		// `MissingArgument` errors have their `line` field set to the beginning of the function
		// parsing.
		let start_line = parser.line();
		let mut args = Vec::with_capacity(function.arity);

		for index in 0..function.arity {
			match parser.parse_expression() {
				Ok(arg) => args.push(arg),
				Err(Error { kind: ErrorKind::EmptySource, .. }) => {
					return Err(
						ErrorKind::MissingArgument { name: function.name.to_owned(), index }
							.error(start_line),
					)
				}
				Err(err) => return Err(err),
			}
		}

		Ok(Some(Self::new(function, args.into())))
	}
}
