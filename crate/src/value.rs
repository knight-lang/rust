// use crate::{Ast, Boolean, Environment, Error, Function, Number, Result, SharedStr, Variable};
use crate::env::{Environment, Variable};
use crate::{Ast, Error, Result, SharedStr};
use std::fmt::{self, Debug, Formatter};

#[cfg(feature = "strict-numbers")]
/// The number type within Knight.
pub type Number = i32;

#[cfg(not(feature = "strict-numbers"))]
/// The number type within Knight.
pub type Number = i64;

/// A Value within Knight.
#[derive(Clone, PartialEq)]
pub enum Value {
	Null,
	Boolean(bool),
	Number(Number),
	SharedStr(SharedStr),
	Variable(Variable),
	Ast(Ast),
}

impl Default for Value {
	#[inline]
	fn default() -> Self {
		Self::Null
	}
}

impl Debug for Value {
	// note we need the custom impl becuase `Null()` and `Identifier(...)` is required by the knight spec.
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Null => write!(f, "Null()"),
			Self::Boolean(boolean) => write!(f, "Boolean({boolean})"),
			Self::Number(number) => write!(f, "Number({number})"),
			Self::SharedStr(text) => write!(f, "SharedStr({text})"),
			Self::Variable(variable) => write!(f, "Variable({})", variable.name()),
			Self::Ast(ast) => write!(f, "{ast:?}"),
		}
	}
}

impl From<()> for Value {
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

impl From<Number> for Value {
	#[inline]
	fn from(number: Number) -> Self {
		Self::Number(number)
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
			Value::Number(number) => Ok(*number != 0),
			Value::SharedStr(text) => Ok(!text.is_empty()),
			_ => Err(Error::NoConversion { from: "Boolean", to: value.typename() }),
		}
	}
}

impl Context for Number {
	fn convert(value: &Value) -> Result<Self> {
		match value {
			Value::Null => Ok(0),
			Value::Boolean(boolean) => Ok(*boolean as Self),
			Value::Number(number) => Ok(*number),
			Value::SharedStr(text) => text.to_number(),
			_ => Err(Error::NoConversion { from: "Number", to: value.typename() }),
		}
	}
}

impl Context for SharedStr {
	fn convert(value: &Value) -> Result<Self> {
		match value {
			Value::Null => Ok(SharedStr::new("null").unwrap()),
			Value::Boolean(boolean) => Ok(SharedStr::new(boolean).unwrap()),
			Value::Number(number) => Ok(SharedStr::new(number).unwrap()),
			Value::SharedStr(text) => Ok(text.clone()),
			_ => Err(Error::NoConversion { from: "SharedStr", to: value.typename() }),
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
			Self::Number(_) => "Number",
			Self::SharedStr(_) => "SharedStr",
			Self::Variable(_) => "Variable",
			Self::Ast(_) => "Ast",
		}
	}

	/// Checks to see if `self` is one of the four builtin types: Null, Boolean, Number, or SharedStr.
	pub const fn is_builtin_type(&self) -> bool {
		matches!(self, Self::Null | Self::Boolean(_) | Self::Number(_) | Self::SharedStr(_))
	}

	/// Converts `self` to a [`bool`] according to the Knight spec.
	pub fn to_bool(&self) -> Result<bool> {
		Context::convert(self)
	}

	/// Converts `self` to a [`Number`] according to the Knight spec.
	pub fn to_number(&self) -> Result<Number> {
		Context::convert(self)
	}

	/// Converts `self` to a [`SharedStr`] according to the Knight spec.
	pub fn to_knstr(&self) -> Result<SharedStr> {
		Context::convert(self)
	}

	pub fn run(&self, env: &mut Environment<'_>) -> Result<Self> {
		match self {
			Self::Variable(variable) => variable.run(),
			Self::Ast(ast) => ast.run(env),
			_ => Ok(self.clone()),
		}
	}

	// pub fn not(&self) -> Result<Self> {
	// 	Ok((!self.to_bool()?).into())
	// }

	// pub fn length(&self) -> Result<usize> {

	// }

	// pub fn compare(&self, rhs: &Self) -> Result<std::cmp::Ordering> {
	// 	match self {
	// 		Self::Number(lhs) => Ok(lhs.cmp(&rhs.to_number()?)),
	// 		Self::Boolean(lhs) => Ok(lhs.cmp(&rhs.to_bool()?)),
	// 		Self::SharedStr(_text) => todo!(),
	// 		_ => Err(Error::TypeError(self.typename())),
	// 	}
	// }
}
