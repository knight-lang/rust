// use crate::{Ast, Boolean, Environment, Error, Function, Number, Result, Text, Variable};
use crate::env::{Environment, Variable};
use crate::{Ast, Error, Result, Text};
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
	Text(Text),
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
			Self::Text(text) => write!(f, "Text({text})"),
			Self::Variable(variable) => write!(f, "Variable({})", variable.name()),
			Self::Ast(ast) => write!(f, "{ast:?}"),
		}
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

impl From<Text> for Value {
	#[inline]
	fn from(text: Text) -> Self {
		Self::Text(text)
	}
}

impl From<Variable> for Value {
	#[inline]
	fn from(variable: Variable) -> Self {
		Self::Variable(variable)
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
			Value::Text(text) => Ok(!text.is_empty()),
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
			Value::Text(text) => text.to_number(),
			_ => Err(Error::NoConversion { from: "Number", to: value.typename() }),
		}
	}
}

impl Context for Text {
	fn convert(value: &Value) -> Result<Self> {
		match value {
			Value::Null => Ok(Text::new("null").unwrap()),
			Value::Boolean(boolean) => Ok(Text::new(boolean).unwrap()),
			Value::Number(number) => Ok(Text::new(number).unwrap()),
			Value::Text(text) => Ok(text.clone()),
			_ => Err(Error::NoConversion { from: "Text", to: value.typename() }),
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
			Self::Text(_) => "Text",
			Self::Variable(_) => "Variable",
			Self::Ast(_) => "Ast",
		}
	}

	/// Converts `self` to a [`bool`] according to the Knight spec.
	pub fn to_bool(&self) -> Result<bool> {
		Context::convert(self)
	}

	/// Converts `self` to a [`Number`] according to the Knight spec.
	pub fn to_number(&self) -> Result<Number> {
		Context::convert(self)
	}

	/// Converts `self` to a [`Text`] according to the Knight spec.
	pub fn to_text(&self) -> Result<Text> {
		Context::convert(self)
	}

	pub fn run(&self, env: &mut Environment<'_>) -> Result<Self> {
		match self {
			Self::Variable(variable) => variable.run(),
			Self::Ast(ast) => ast.run(env),
			_ => Ok(self.clone()),
		}
	}
}
