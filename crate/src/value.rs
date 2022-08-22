// use crate::{Ast, Boolean, Environment, Error, Function, Integer, Result, SharedText, Variable};
use crate::env::Environment;
use crate::{Ast, Error, Integer, Result, SharedText, Variable};
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
	SharedText(SharedText),

	/// Represents a variable.
	Variable(Variable),

	/// Represents a block of code.
	Ast(Ast),

	#[cfg(feature = "arrays")]
	List(crate::List),
}
#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Value: Send, Sync);

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
			Self::SharedText(text) => write!(f, "Text({text})"),
			Self::Variable(variable) => write!(f, "Identifier({})", variable.name()),
			Self::Ast(ast) => write!(f, "{ast:?}"),
			#[cfg(feature = "arrays")]
			Self::List(list) => Debug::fmt(&list, f),
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

impl From<SharedText> for Value {
	#[inline]
	fn from(text: SharedText) -> Self {
		Self::SharedText(text)
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

#[cfg(feature = "arrays")]
impl From<crate::List> for Value {
	#[inline]
	fn from(list: crate::List) -> Self {
		Self::List(list)
	}
}

pub trait Context: Sized {
	fn convert(value: &Value) -> Result<Self>;
}

impl Context for bool {
	fn convert(value: &Value) -> Result<Self> {
		match *value {
			Value::Null => Ok(false),
			Value::Boolean(boolean) => Ok(boolean),
			Value::Integer(number) => Ok(number != 0),
			Value::SharedText(ref text) => Ok(!text.is_empty()),
			#[cfg(feature = "arrays")]
			Value::List(ref list) => Ok(!list.is_empty()),
			_ => Err(Error::NoConversion { to: "Boolean", from: value.typename() }),
		}
	}
}

impl Context for Integer {
	fn convert(value: &Value) -> Result<Self> {
		match *value {
			Value::Null => Ok(0),
			Value::Boolean(boolean) => Ok(boolean as Self),
			Value::Integer(number) => Ok(number),
			Value::SharedText(ref text) => text.to_integer(),
			#[cfg(feature = "arrays")]
			Value::List(ref list) => Ok(list.len() as Self),
			_ => Err(Error::NoConversion { to: "Integer", from: value.typename() }),
		}
	}
}

impl Context for SharedText {
	fn convert(value: &Value) -> Result<Self> {
		match *value {
			Value::Null => Ok("null".try_into().unwrap()),
			Value::Boolean(boolean) => Ok(SharedText::new(boolean).unwrap()),
			Value::Integer(number) => Ok(SharedText::new(number).unwrap()),
			Value::SharedText(ref text) => Ok(text.clone()),
			#[cfg(feature = "arrays")]
			Value::List(ref list) => list.to_text(),
			_ => Err(Error::NoConversion { to: "String", from: value.typename() }),
		}
	}
}

#[cfg(feature = "arrays")]
impl Context for crate::List {
	fn convert(value: &Value) -> Result<Self> {
		match *value {
			Value::Null => Ok(Self::default()),
			Value::Boolean(boolean) => todo!(),
			Value::Integer(mut number) => {
				if number == 0 {
					return Ok(vec![0.into()].into());
				}

				// TODO: when log10 is finalized, add it in.
				let mut list = Vec::new();

				let is_negative = if number < 0 {
					number = -number; // TODO: checked negation.
					true
				} else {
					false
				};

				while number != 0 {
					list.push(Value::from(number % 10));
					number /= 10;
				}

				if is_negative {
					list.push((-1).into());
				}

				list.reverse();

				Ok(list.into())
			}
			Value::SharedText(ref text) => Ok(text
				.chars()
				.map(|c| Value::from(SharedText::try_from(c.to_string()).unwrap()))
				.collect()),
			Value::List(ref list) => Ok(list.clone()),
			_ => Err(Error::NoConversion { to: "List", from: value.typename() }),
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
			Self::SharedText(_) => "SharedText",
			Self::Variable(_) => "Variable",
			Self::Ast(_) => "Ast",
			#[cfg(feature = "arrays")]
			Self::List(_) => "List",
		}
	}

	/// Converts `self` to a [`bool`] according to the Knight spec.
	pub fn to_bool(&self) -> Result<bool> {
		Context::convert(self)
	}

	/// Converts `self` to an [`Integer`] according to the Knight spec.
	pub fn to_integer(&self) -> Result<Integer> {
		Context::convert(self)
	}

	/// Converts `self` to a [`SharedText`] according to the Knight spec.
	pub fn to_text(&self) -> Result<SharedText> {
		Context::convert(self)
	}

	#[cfg(feature = "arrays")]
	pub fn to_array(&self) -> Result<crate::List> {
		Context::convert(self)
	}

	/// Executes the value.
	pub fn run(&self, env: &mut Environment) -> Result<Self> {
		match self {
			Self::Variable(variable) => {
				variable.fetch().ok_or_else(|| Error::UndefinedVariable(variable.name().clone()))
			}
			Self::Ast(ast) => ast.run(env),
			_ => Ok(self.clone()),
		}
	}
}
