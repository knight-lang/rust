use crate::{Mutable, RefCount, Text, TextSlice, Value};
use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Represents a variable within Knight.
#[derive(Clone)]
pub struct Variable(RefCount<(Text, Mutable<Option<Value>>)>);

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Variable: Send, Sync);

impl Debug for Variable {
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

impl Eq for Variable {}
impl PartialEq for Variable {
	/// Checks to see if two variables are equal.
	///
	/// This'll just check to see if their names are equivalent. Techincally, this means that
	/// two variables with the same name, but derived from different [`Environment`]s will end up
	/// being the same.
	#[inline]
	fn eq(&self, rhs: &Self) -> bool {
		self.name() == rhs.name()
	}
}

impl Borrow<TextSlice> for Variable {
	#[inline]
	fn borrow(&self) -> &TextSlice {
		self.name()
	}
}

impl Hash for Variable {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name().hash(state);
	}
}

/// Indicates that a a variable name was illegal.
///
/// This is only ever returned if the `verify-variable-names` feature is is enabled.
#[derive(Debug, PartialEq, Eq)]
pub enum IllegalVariableName {
	/// The name was empty
	Empty,

	/// The name was too long.
	TooLong(usize),

	/// The name had an illegal character at the beginning.
	IllegalStartingChar(char),

	/// The name had an illegal character in the middle.
	IllegalBodyChar(char),
}

impl Display for IllegalVariableName {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			Self::Empty => write!(f, "empty variable name supplied"),
			Self::TooLong(count) => write!(f, "variable name was too long ({count} > {MAX_NAME_LEN})"),
			Self::IllegalStartingChar(chr) => write!(f, "variable names cannot start with {chr:?}"),
			Self::IllegalBodyChar(chr) => write!(f, "variable names cannot include with {chr:?}"),
		}
	}
}

impl std::error::Error for IllegalVariableName {}

/// Maximum length a name can have when `verify-variable-names` is enabled.
pub const MAX_NAME_LEN: usize = 255;

/// Check to see if `name` is a valid variable name. Unless `verify-variable-names` is enabled, this
/// will always return `Ok(())`.
pub fn validate_name(name: &TextSlice) -> Result<(), IllegalVariableName> {
	if cfg!(not(feature = "verify-variable-names")) {
		return Ok(());
	}

	use crate::parser::{is_lower, is_numeric};

	if MAX_NAME_LEN < name.len() {
		return Err(IllegalVariableName::TooLong(name.len()));
	}

	let first = name.chars().next().ok_or(IllegalVariableName::Empty)?;
	if !is_lower(first) {
		return Err(IllegalVariableName::IllegalStartingChar(first));
	}

	if let Some(bad) = name.chars().find(|&c| !is_lower(c) && !is_numeric(c)) {
		return Err(IllegalVariableName::IllegalBodyChar(bad));
	}

	Ok(())
}

impl Variable {
	/// Creates a new `Variable`.
	#[must_use]
	pub fn new(name: Text) -> Result<Self, IllegalVariableName> {
		validate_name(&name)?;

		Ok(Self((name, None.into()).into()))
	}

	/// Fetches the name of the variable.
	#[must_use]
	#[inline]
	pub fn name(&self) -> &Text {
		&(self.0).0
	}

	/// Assigns a new value to the variable, returning whatever the previous value was.
	pub fn assign(&self, new: Value) -> Option<Value> {
		(self.0).1.write().replace(new)
	}

	/// Fetches the last value assigned to `self`, returning `None` if we haven't been assigned to yet.
	#[must_use]
	pub fn fetch(&self) -> Option<Value> {
		(self.0).1.read().clone()
	}
}
