use crate::env::Environment;
use crate::value::{
	Boolean, Integer, List, NamedType, Null, Runnable, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::{Ast, Error, Result, Variable};

/// A Value within Knight.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Value<'e> {
	#[default]
	/// Represents the `NULL` value.
	Null,

	/// Represents the `TRUE` and `FALSE` values.
	Boolean(Boolean),

	/// Represents integers.
	Integer(Integer),

	/// Represents a string.
	Text(Text),

	/// Represents a list of [`Value`]s.
	List(List<'e>),

	/// Represents a variable.
	Variable(Variable<'e>),

	/// Represents a block of code.
	Ast(Ast<'e>),
}

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(Value<'_>: Send, Sync);

impl From<Null> for Value<'_> {
	#[inline]
	fn from(_: Null) -> Self {
		Self::Null
	}
}

impl From<Boolean> for Value<'_> {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		Self::Boolean(boolean)
	}
}

impl From<Integer> for Value<'_> {
	#[inline]
	fn from(number: Integer) -> Self {
		Self::Integer(number)
	}
}

impl From<Text> for Value<'_> {
	#[inline]
	fn from(text: Text) -> Self {
		Self::Text(text)
	}
}

impl From<crate::value::text::Character> for Value<'_> {
	#[inline]
	fn from(character: crate::value::text::Character) -> Self {
		Self::Text(Text::from(character))
	}
}

impl<'e> From<Variable<'e>> for Value<'e> {
	#[inline]
	fn from(variable: Variable<'e>) -> Self {
		Self::Variable(variable)
	}
}

impl<'e> From<Ast<'e>> for Value<'e> {
	#[inline]
	fn from(inp: Ast<'e>) -> Self {
		Self::Ast(inp)
	}
}

impl<'e> From<List<'e>> for Value<'e> {
	#[inline]
	fn from(list: List<'e>) -> Self {
		Self::List(list)
	}
}

impl<'e> Value<'e> {
	/// Fetch the type's name.
	#[must_use = "getting the type name by itself does nothing."]
	pub const fn typename(&self) -> &'static str {
		match self {
			Self::Null => Null::TYPENAME,
			Self::Boolean(_) => Boolean::TYPENAME,
			Self::Integer(_) => Integer::TYPENAME,
			Self::Text(_) => Text::TYPENAME,
			Self::List(_) => List::TYPENAME,
			Self::Ast(_) => Ast::TYPENAME,
			Self::Variable(_) => Variable::TYPENAME,
		}
	}
}

impl ToBoolean for Value<'_> {
	fn to_boolean(&self) -> Result<Boolean> {
		match *self {
			Self::Null => Null.to_boolean(),
			Self::Boolean(boolean) => boolean.to_boolean(),
			Self::Integer(integer) => integer.to_boolean(),
			Self::Text(ref text) => text.to_boolean(),
			Self::List(ref list) => list.to_boolean(),
			_ => Err(Error::NoConversion { to: Boolean::TYPENAME, from: self.typename() }),
		}
	}
}

impl ToInteger for Value<'_> {
	fn to_integer(&self) -> Result<Integer> {
		match *self {
			Self::Null => Null.to_integer(),
			Self::Boolean(boolean) => boolean.to_integer(),
			Self::Integer(integer) => integer.to_integer(),
			Self::Text(ref text) => text.to_integer(),
			Self::List(ref list) => list.to_integer(),
			_ => Err(Error::NoConversion { to: Integer::TYPENAME, from: self.typename() }),
		}
	}
}

impl ToText for Value<'_> {
	fn to_text(&self) -> Result<Text> {
		match *self {
			Self::Null => Null.to_text(),
			Self::Boolean(boolean) => boolean.to_text(),
			Self::Integer(integer) => integer.to_text(),
			Self::Text(ref text) => text.to_text(),
			Self::List(ref list) => list.to_text(),
			_ => Err(Error::NoConversion { to: Text::TYPENAME, from: self.typename() }),
		}
	}
}

impl<'e> ToList<'e> for Value<'e> {
	fn to_list(&self) -> Result<List<'e>> {
		match *self {
			Self::Null => Null.to_list(),
			Self::Boolean(boolean) => boolean.to_list(),
			Self::Integer(integer) => integer.to_list(),
			Self::Text(ref text) => text.to_list(),
			Self::List(ref list) => list.to_list(),
			_ => Err(Error::NoConversion { to: List::TYPENAME, from: self.typename() }),
		}
	}
}

impl<'e> Runnable<'e> for Value<'e> {
	fn run(&self, env: &mut Environment<'e>) -> Result<Self> {
		match self {
			Self::Variable(variable) => variable.run(env),
			Self::Ast(ast) => ast.run(env),
			_ => Ok(self.clone()),
		}
	}
}
