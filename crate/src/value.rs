// use crate::{Ast, Boolean, Environment, Error, Function, Integer, Result, SharedStr, Variable};
use crate::env::{Environment, Variable};
use crate::{Ast, Error, Integer, Result, SharedStr};
use std::fmt::{self, Debug, Formatter};

/// A Value within Knight.
#[derive(Clone, PartialEq)]
pub enum Value {
	/// Represents the `NULL` value.
	Null,

	/// Represents the `TRUE` and `FALSE` values.
	Boolean(bool),

	/// Represents integers.
	Integer(Integer),

	/// Represents a string.
	SharedStr(SharedStr),

	/// Represents a variable.
	Variable(Variable),

	/// Represents a block of code.
	Ast(Ast),
}

impl Default for Value {
	#[inline]
	fn default() -> Self {
		Self::Null
	}
}

impl Debug for Value {
	// note we need the custom impl becuase `Null()` and `Identifier(...)` are needed by the tester.
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Null => write!(f, "Null()"),
			Self::Boolean(boolean) => write!(f, "Boolean({boolean})"),
			Self::Integer(number) => write!(f, "Integer({number})"),
			Self::SharedStr(text) => write!(f, "Text({text})"),
			Self::Variable(variable) => write!(f, "Identifier({})", variable.name()),
			Self::Ast(ast) => write!(f, "{ast:?}"),
		}
	}
}

impl From<()> for Value {
	#[inline]
	fn from(_: ()) -> Self {
		Self::Null
	}
}

impl From<bool> for Value {
	#[inline]
	fn from(boolean: bool) -> Self {
		Self::Boolean(boolean)
	}
}

impl From<Integer> for Value {
	#[inline]
	fn from(number: Integer) -> Self {
		Self::Integer(number)
	}
}

impl From<SharedStr> for Value {
	#[inline]
	fn from(text: SharedStr) -> Self {
		Self::SharedStr(text)
	}
}

impl From<Variable> for Value {
	#[inline]
	fn from(variable: Variable) -> Self {
		Self::Variable(variable)
	}
}

impl From<Ast> for Value {
	#[inline]
	fn from(inp: Ast) -> Self {
		Self::Ast(inp)
	}
}

pub trait Context: Sized {
	fn convert(value: &Value) -> Result<Self>;
}

impl Context for bool {
	fn convert(value: &Value) -> Result<Self> {
		match value {
			Value::Null => Ok(false),
			Value::Boolean(boolean) => Ok(*boolean),
			Value::Integer(number) => Ok(*number != 0),
			Value::SharedStr(text) => Ok(!text.is_empty()),
			_ => Err(Error::NoConversion { to: "Boolean", from: value.typename() }),
		}
	}
}

impl Context for Integer {
	fn convert(value: &Value) -> Result<Self> {
		match value {
			Value::Null => Ok(0),
			Value::Boolean(boolean) => Ok(*boolean as Self),
			Value::Integer(number) => Ok(*number),
			Value::SharedStr(text) => text.to_integer(),
			_ => Err(Error::NoConversion { to: "Integer", from: value.typename() }),
		}
	}
}

impl Context for SharedStr {
	fn convert(value: &Value) -> Result<Self> {
		match value {
			Value::Null => Ok(SharedStr::new("null").unwrap()),
			Value::Boolean(boolean) => Ok(SharedStr::new(boolean).unwrap()),
			Value::Integer(number) => Ok(SharedStr::new(number).unwrap()),
			Value::SharedStr(text) => Ok(text.clone()),
			_ => Err(Error::NoConversion { to: "SharedStr", from: value.typename() }),
		}
	}
}

impl Value {
	/// Fetch the type's name.
	#[must_use = "getting the type name by itself does nothing."]
	pub const fn typename(&self) -> &'static str {
		match self {
			Self::Null => "Null",
			Self::Boolean(_) => "Boolean",
			Self::Integer(_) => "Integer",
			Self::SharedStr(_) => "SharedStr",
			Self::Variable(_) => "Variable",
			Self::Ast(_) => "Ast",
		}
	}

	/// Checks to see if `self` is one of the four builtin types: [`Null`], [`Boolean`], [`Integer`],
	/// or [`SharedStr`].
	pub const fn is_builtin_type(&self) -> bool {
		matches!(self, Self::Null | Self::Boolean(_) | Self::Integer(_) | Self::SharedStr(_))
	}

	/// Converts `self` to a [`bool`] according to the Knight spec.
	pub fn to_bool(&self) -> Result<bool> {
		Context::convert(self)
	}

	/// Converts `self` to an [`Integer`] according to the Knight spec.
	pub fn to_integer(&self) -> Result<Integer> {
		Context::convert(self)
	}

	/// Converts `self` to a [`SharedStr`] according to the Knight spec.
	pub fn to_knstr(&self) -> Result<SharedStr> {
		Context::convert(self)
	}

	/// Executes the value.
	pub fn run(&self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Variable(variable) => variable.run(),
			Self::Ast(ast) => ast.run(env),
			_ => Ok(self.clone()),
		}
	}
}
