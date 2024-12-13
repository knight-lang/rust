//! Types relating to [`Variable`]s.

use crate::containers::{Mutable, RefCount};
use crate::env::{Environment, Flags};
use crate::parse::{self, Parsable, Parser};
use crate::value::{NamedType, Runnable, Text, TextSlice, Value};
use crate::{Error, Result};
use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Represents a variable within Knight.
///
/// You'll never create variables directly; Instead, use [`Environment::lookup`].
#[derive(Clone)]
pub struct Variable(RefCount<Inner>);

struct Inner {
	name: Text,
	value: Mutable<Option<Value>>,
}

impl Debug for Variable {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Variable")
				.field("name", &self.name())
				.field("value", &self.0.value)
				.finish()
		} else {
			f.debug_tuple("Variable").field(self.name()).finish()
		}
	}
}

impl Eq for Variable {}
impl PartialEq for Variable {
	/// Checks to see if two variables are pointing to the _exact same object_
	#[inline]
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl Borrow<TextSlice> for Variable {
	/// Borrows the [name](Variable::name) of the variable.
	#[inline]
	fn borrow(&self) -> &TextSlice {
		self.name()
	}
}

impl Hash for Variable {
	/// Hashes the [name](Variable::name) of the variable.
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name().hash(state);
	}
}

impl NamedType for Variable {
	const TYPENAME: &'static str = "Variable";
}

/// Indicates that a a variable name was illegal.
///
/// While the enum itself is not feature gated, every one of its variants requires `compliance` to
/// be enabled. This means that if `compliance` isn't enabled, then it's impossible to ever
/// construct this type.
///
/// NOTE: Technically all the variants other than `TooLong` aren't able to be created without
/// extensions in this implementation. The parser ensures that all values passed to
/// [`Environment::lookup`] are nonempty and has a valid first and remaining characters. However,
/// with the [`VALUE`](crate::functions::VALUE) extension (as well as some other things, like
/// [assigning to lists](crate::env::flags::AssignTo::list)), it's possible for these to be created.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum IllegalVariableName {
	/// The name was empty.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	Empty,

	/// The name was too long.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	TooLong(usize),

	/// The name had an illegal character at the beginning.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	IllegalStartingChar(char),

	/// The name had an illegal character in the middle.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	IllegalBodyChar(char),
}

impl std::error::Error for IllegalVariableName {}

impl Display for IllegalVariableName {
	fn fmt(&self, #[allow(unused)] f: &mut Formatter) -> fmt::Result {
		match *self {
			#[cfg(feature = "compliance")]
			Self::Empty => write!(f, "empty variable name supplied"),
			#[cfg(feature = "compliance")]
			Self::TooLong(count) => write!(f, "variable name was too long ({count} > {MAX_NAME_LEN})"),
			#[cfg(feature = "compliance")]
			Self::IllegalStartingChar(chr) => write!(f, "variable names cannot start with {chr:?}"),
			#[cfg(feature = "compliance")]
			Self::IllegalBodyChar(chr) => write!(f, "variable names cannot include with {chr:?}"),
		}
	}
}

/// Maximum length a name can have when [`verify_variable_names`](
/// crate::env::flags::Compliance::verify_variable_names) is enabled.
pub const MAX_NAME_LEN: usize = 127;

impl Variable {
	#[cfg(feature = "compliance")]
	fn validate_name(
		name: &TextSlice,
		flags: &Flags,
	) -> std::result::Result<(), IllegalVariableName> {
		if MAX_NAME_LEN < name.len() {
			return Err(IllegalVariableName::TooLong(name.len()));
		}

		match name.head() {
			Some('a'..='z' | '_') => {}
			Some(first) if !flags.compliance.knight_encoding && first.is_lowercase() => {}
			Some(first) => return Err(IllegalVariableName::IllegalStartingChar(first)),
			None => return Err(IllegalVariableName::Empty),
		}

		if let Some(bad) = name.chars().find(|&chr| {
			if flags.compliance.knight_encoding {
				return !matches!(chr, 'a'..='z' | '_' | '0'..='9');
			}

			!chr.is_lowercase() && chr != '_' && !chr.is_numeric()
		}) {
			return Err(IllegalVariableName::IllegalBodyChar(bad));
		}

		Ok(())
	}

	pub(crate) fn new(name: Text, flags: &Flags) -> std::result::Result<Self, IllegalVariableName> {
		#[cfg(feature = "compliance")]
		if flags.compliance.verify_variable_names {
			Self::validate_name(&name, flags)?;
		}

		let _ = flags;
		Ok(Self(Inner { name, value: None.into() }.into()))
	}

	/// Fetches the name of the variable.
	#[must_use]
	#[inline]
	pub fn name(&self) -> &Text {
		&self.0.name
	}

	/// Assigns a new value to the variable, returning whatever the previous value was.
	#[inline]
	pub fn assign(&self, new: Value) -> Option<Value> {
		(self.0).value.write().replace(new)
	}

	/// Fetches the last value assigned to `self`, returning `None` if it haven't been assigned yet.
	#[must_use = "fetching the value of a variable does nothing on its own"]
	#[inline]
	pub fn fetch(&self) -> Option<Value> {
		(self.0).value.read().clone()
	}
}

impl Runnable for Variable {
	/// [Fetches](Self::fetch) the last assigned value, or returns [`Error::UndefinedVariable`] if
	/// it was never assigned to.
	fn run(&self, env: &mut Environment) -> Result<Value> {
		let _ = env;

		match self.fetch() {
			Some(value) => Ok(value),

			#[cfg(feature = "iffy-extensions")]
			None if env.flags().extensions.iffy.unassigned_variables_default_to_null => Ok(Value::Null),

			None => Err(Error::UndefinedVariable(self.name().to_string())),
		}
	}
}

impl Parsable for Variable {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_>) -> parse::Result<Option<Self>> {
		if parser.peek().map_or(false, |c| !c.is_lowercase() && c != '_') {
			return Ok(None);
		}

		let Some(ident) = parser.take_while(|c| c.is_lowercase() || c == '_' || c.is_numeric())
		else {
			return Ok(None);
		};

		match parser.env().lookup(ident) {
			Ok(value) => Ok(Some(value)),
			Err(err) => match err {
				// When there's no compliance issues, there'll be nothing to match.
				#[cfg(feature = "compliance")]
				err => Err(parser.error(parse::ErrorKind::IllegalVariableName(err))),
			},
		}
	}
}
