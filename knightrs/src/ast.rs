use crate::containers::RefCount;
use crate::env::Environment;
use crate::function::Function;
use crate::parse::{self, Parsable, Parser};
use crate::value::{Runnable, Value};
use crate::Result;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// [`Ast`]s represent functions and their arguments.
#[derive(Debug, Clone)]
pub struct Ast(RefCount<Inner>);

#[derive(Debug)]
struct Inner {
	function: Function,
	args: Box<[Value]>,
}

impl Eq for Ast {}
impl PartialEq for Ast {
	/// Two `Ast`s are equal only if they point to the exact same data.
	#[inline]
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl Hash for Ast {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(RefCount::as_ptr(&self.0) as usize).hash(state);
	}
}

impl crate::value::NamedType for Ast {
	const TYPENAME: &'static str = "Ast";
}

impl Ast {
	/// Creates a new `Ast` from the given arguments.
	///
	/// # Panics
	/// Panics if `args.len()` isn't equal to `function.arity`.
	#[must_use]
	#[inline]
	pub fn new(function: Function, args: Box<[Value]>) -> Self {
		assert_eq!(args.len(), function.arity());

		Self(Inner { function, args }.into())
	}

	/// Gets the function associated with the ast.
	#[must_use]
	#[inline]
	pub fn function(&self) -> &Function {
		&self.0.function
	}

	/// Gets the args associated with the ast.
	#[must_use]
	#[inline]
	pub fn args(&self) -> &[Value] {
		&self.0.args
	}
}

impl Runnable for Ast {
	#[inline]
	fn run(&self, env: &mut Environment<'_>) -> Result<Value> {
		self.function().run(self.args(), env)
	}
}

impl Parsable for Ast {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_>) -> parse::Result<Option<Self>> {
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
