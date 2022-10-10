use crate::env::Flags;
use crate::parse::{self, Parsable, Parser};
use crate::value::{Runnable, Text, TextSlice, Value};
use crate::{Environment, Error, Mutable, RefCount, Result};
use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Represents a variable within Knight.
///
/// You'll never create variables directly; Instead, use [`Environment::lookup`].
// FIXME: You can memory leak via `= a (B a)` (and also `= a (B + a 1)`, etc.)
#[derive(Clone)]
pub struct Variable<'e>(RefCount<Inner<'e>>);

struct Inner<'e> {
	name: Text,
	value: Mutable<Option<Value<'e>>>,
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Variable<'_>: Send, Sync);

impl Debug for Variable<'_> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		if f.alternate() {
			f.debug_struct("Variable")
				.field("name", &self.name())
				.field("value", &self.fetch())
				.finish()
		} else {
			write!(f, "Variable({})", self.name())
		}
	}
}

impl Eq for Variable<'_> {}
impl PartialEq for Variable<'_> {
	/// Checks to see if two variables are equal.
	///
	/// This checks to see if the two variables are pointing to the _exact same object_.
	#[inline]
	fn eq(&self, rhs: &Self) -> bool {
		RefCount::ptr_eq(&self.0, &rhs.0)
	}
}

impl Borrow<TextSlice> for Variable<'_> {
	/// Borrows the [`name`](Variable::name) of the variable.
	#[inline]
	fn borrow(&self) -> &TextSlice {
		self.name()
	}
}

impl Hash for Variable<'_> {
	/// Hashes the [`name`](Variable::name) of the variable.
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name().hash(state);
	}
}

impl crate::value::NamedType for Variable<'_> {
	const TYPENAME: &'static str = "Variable";
}

/// Indicates that a a variable name was illegal.
///
/// While the enum itself is not feature gated, every one of its variants requires `compliance` to
/// be enabled. This means that if `compliance` isn't enabled, then it's impossible to ever
/// construct this type.
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
	IllegalStartingChar(crate::value::text::Character),

	/// The name had an illegal character in the middle.
	#[cfg(feature = "compliance")]
	#[cfg_attr(docsrs, doc(cfg(feature = "compliance")))]
	IllegalBodyChar(crate::value::text::Character),
}

impl std::error::Error for IllegalVariableName {}

impl Display for IllegalVariableName {
	fn fmt(&self, #[allow(unused)] f: &mut Formatter) -> fmt::Result {
		match *self {
			#[cfg(feature = "compliance")]
			Self::Empty => write!(f, "empty variable name supplied"),

			#[cfg(feature = "compliance")]
			Self::TooLong(count) => {
				write!(f, "variable name was too long ({count} > {})", Variable::MAX_NAME_LEN)
			}

			#[cfg(feature = "compliance")]
			Self::IllegalStartingChar(chr) => write!(f, "variable names cannot start with {chr:?}"),

			#[cfg(feature = "compliance")]
			Self::IllegalBodyChar(chr) => write!(f, "variable names cannot include with {chr:?}"),
		}
	}
}

impl<'e> Variable<'e> {
	/// Maximum length a name can have when [`verify_variable_names`](
	/// crate::env::flags::ComplianceFlags::verify_variable_names) is enabled.
	pub const MAX_NAME_LEN: usize = 127;

	#[cfg(feature = "compliance")]
	fn validate_name(name: &TextSlice) -> std::result::Result<(), IllegalVariableName> {
		if Self::MAX_NAME_LEN < name.len() {
			return Err(IllegalVariableName::TooLong(name.len()));
		}

		let first = name.chars().next().ok_or(IllegalVariableName::Empty)?;
		if !first.is_lower() {
			return Err(IllegalVariableName::IllegalStartingChar(first));
		}

		if let Some(bad) = name.chars().find(|&c| !c.is_lower() && !c.is_numeric()) {
			return Err(IllegalVariableName::IllegalBodyChar(bad));
		}

		Ok(())
	}

	pub(crate) fn new(name: Text, flags: &Flags) -> std::result::Result<Self, IllegalVariableName> {
		#[cfg(feature = "compliance")]
		if flags.compliance.verify_variable_names {
			Self::validate_name(&name)?;
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
	pub fn assign(&self, new: Value<'e>) -> Option<Value<'e>> {
		(self.0).value.write().replace(new)
	}

	/// Fetches the last value assigned to `self`, returning `None` if it haven't been assigned yet.
	#[must_use]
	pub fn fetch(&self) -> Option<Value<'e>> {
		(self.0).value.read().clone()
	}
}

impl<'e> Runnable<'e> for Variable<'e> {
	fn run(&self, _env: &mut Environment) -> Result<Value<'e>> {
		self.fetch().ok_or_else(|| Error::UndefinedVariable(self.name().clone()))
	}
}

impl<'e> Parsable<'e> for Variable<'e> {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, 'e>) -> parse::Result<Option<Self>> {
		let Some(identifier) = parser.take_while(|chr| chr.is_lower() || chr.is_numeric()) else {
			return Ok(None);
		};

		match parser.env().lookup(identifier) {
			Ok(value) => Ok(Some(value)),
			Err(err) => match err {
				// When there's no extensions, there'll be nothing to match.
				#[cfg(feature = "extensions")]
				err => Err(parser.error(parse::ErrorKind::IllegalVariableName(err))),
			},
		}
	}
}
