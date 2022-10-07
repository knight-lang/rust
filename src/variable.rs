use crate::env::Options;
use crate::value::text::Encoding;
use crate::value::{Runnable, Text, TextSlice, Value};
use crate::IntType;
use crate::{Environment, Error};
use std::borrow::Borrow;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

/// Represents a variable within Knight.
pub struct Variable<'e, E: Encoding, I: IntType>(Arc<Inner<'e, E, I>>);

struct Inner<'e, E: Encoding, I: IntType> {
	name: Text<E>,
	value: Mutex<Option<Value<'e, E, I>>>,
}

impl<E: Encoding, I: IntType> Clone for Variable<'_, E, I> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<E: Encoding, I: IntType> Debug for Variable<'_, E, I> {
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

impl<E: Encoding, I: IntType> Eq for Variable<'_, E, I> {}
impl<E: Encoding, I: IntType> PartialEq for Variable<'_, E, I> {
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

impl<E: Encoding, I: IntType> Borrow<TextSlice<E>> for Variable<'_, E, I> {
	#[inline]
	fn borrow(&self) -> &TextSlice<E> {
		self.name()
	}
}

impl<E: Encoding, I: IntType> Hash for Variable<'_, E, I> {
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
			Self::TooLong(len) => write!(f, "variable too long ({len} > {MAX_LEN})",),
			Self::IllegalStartingChar(chr) => write!(f, "variable names cannot start with {chr:?}"),
			Self::IllegalBodyChar(chr) => write!(f, "variable names cannot include with {chr:?}"),
		}
	}
}

impl std::error::Error for IllegalVariableName {}

/// Check to see if `name` is a valid variable name. Unless `verify-variable-names` is enabled, this
/// will always return `Ok(())`.
fn validate_name<E: Encoding>(
	name: &TextSlice<E>,
	options: &Options,
) -> Result<(), IllegalVariableName> {
	if !options.compliance.variable_name {
		return Ok(());
	}

	if MAX_LEN < name.len() {
		return Err(IllegalVariableName::TooLong(name.len()));
	}

	let first = name.chars().next().ok_or(IllegalVariableName::Empty)?;
	if !first.is_lowercase() {
		return Err(IllegalVariableName::IllegalStartingChar(first.into()));
	}

	if let Some(bad) = name.chars().find(|&c| !c.is_lowercase() && !c.is_numeric()) {
		return Err(IllegalVariableName::IllegalBodyChar(bad.into()));
	}

	Ok(())
}

impl<'e, E: Encoding, I: IntType> Variable<'e, E, I> {
	/// Creates a new `Variable`.
	pub fn new(name: Text<E>, options: &Options) -> Result<Self, IllegalVariableName> {
		validate_name::<E>(&name, options)?;

		Ok(Self(Inner { name, value: None.into() }.into()))
	}
}

/// Maximum length a name can have when `verify-variable-names` is enabled.
pub const MAX_LEN: usize = 127;

impl<'e, E: Encoding, I: IntType> Variable<'e, E, I> {
	/// Fetches the name of the variable.
	#[must_use]
	#[inline]
	pub fn name(&self) -> &Text<E> {
		&self.0.name
	}

	/// Assigns a new value to the variable, returning whatever the previous value was.
	pub fn assign(&self, new: Value<'e, E, I>) -> Option<Value<'e, E, I>> {
		self.0.value.lock().unwrap().replace(new)
	}

	/// Fetches the last value assigned to `self`, returning `None` if we haven't been assigned to yet.
	#[must_use]
	pub fn fetch(&self) -> Option<Value<'e, E, I>> {
		self.0.value.lock().unwrap().clone()
	}
}

impl<'e, E: Encoding, I: IntType> Runnable<'e, E, I> for Variable<'e, E, I> {
	fn run(&self, _env: &mut Environment<'e, E, I>) -> crate::Result<Value<'e, E, I>> {
		self.fetch().ok_or_else(|| Error::UndefinedVariable(self.name().to_string()))
	}
}
