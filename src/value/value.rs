use crate::env::{Environment, Options};
use crate::value::integer::IntType;
use crate::value::text::{Character, Encoding};
use crate::value::{
	Boolean, Integer, List, NamedType, Null, Runnable, Text, ToBoolean, ToInteger, ToList, ToText,
};
use crate::{Ast, Error, Result, Variable};
use std::fmt::{self, Debug, Formatter};

/// A Value within Knight.
pub enum Value<'e, E, I: IntType> {
	/// Represents the `NULL` value.
	Null,

	/// Represents the `TRUE` and `FALSE` values.
	Boolean(Boolean),

	/// Represents integers.
	Integer(Integer<I>),

	/// Represents a string.
	Text(Text<E>),

	/// Represents a list of [`Value`]s.
	List(List<'e, E, I>),

	/// Represents a variable.
	Variable(Variable<'e, E, I>),

	/// Represents a block of code.
	Ast(Ast<'e, E, I>),
}

impl<E, I: IntType> Default for Value<'_, E, I> {
	#[inline]
	fn default() -> Self {
		Self::Null
	}
}

impl<E, I: IntType> Clone for Value<'_, E, I> {
	fn clone(&self) -> Self {
		match self {
			Self::Null => Self::Null,
			Self::Boolean(boolean) => Self::Boolean(boolean.clone()),
			Self::Integer(integer) => Self::Integer(integer.clone()),
			Self::Text(text) => Self::Text(text.clone()),
			Self::List(list) => Self::List(list.clone()),
			Self::Variable(variable) => Self::Variable(variable.clone()),
			Self::Ast(ast) => Self::Ast(ast.clone()),
		}
	}
}

impl<E, I: IntType> PartialEq for Value<'_, E, I> {
	fn eq(&self, rhs: &Self) -> bool {
		match (self, rhs) {
			(Self::Null, Self::Null) => true,
			(Self::Boolean(lhs), Self::Boolean(rhs)) => lhs == rhs,
			(Self::Integer(lhs), Self::Integer(rhs)) => lhs == rhs,
			(Self::Text(lhs), Self::Text(rhs)) => lhs == rhs,
			(Self::List(lhs), Self::List(rhs)) => lhs == rhs,
			(Self::Variable(lhs), Self::Variable(rhs)) => lhs == rhs,
			(Self::Ast(lhs), Self::Ast(rhs)) => lhs == rhs,
			_ => false,
		}
	}
}

impl<E, I: IntType> Debug for Value<'_, E, I> {
	// note we need the custom impl becuase `Null()` and `Identifier(...)` are needed by the tester.
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Boolean(boolean) => write!(f, "{boolean}"),
			Self::Integer(number) => write!(f, "{number}"),
			Self::Text(text) => write!(f, "{:?}", &**text),
			Self::Variable(variable) => write!(f, "{variable:?}"),
			Self::Ast(ast) => Debug::fmt(&ast, f),
			Self::List(list) => Debug::fmt(&list, f),
		}
	}
}

impl<E, I: IntType> From<Null> for Value<'_, E, I> {
	#[inline]
	fn from(_: Null) -> Self {
		Self::Null
	}
}

impl<E, I: IntType> From<Boolean> for Value<'_, E, I> {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		Self::Boolean(boolean)
	}
}

impl<E, I: IntType> From<Integer<I>> for Value<'_, E, I> {
	#[inline]
	fn from(number: Integer<I>) -> Self {
		Self::Integer(number)
	}
}

impl<E, I: IntType> From<Text<E>> for Value<'_, E, I> {
	#[inline]
	fn from(text: Text<E>) -> Self {
		Self::Text(text)
	}
}

impl<E, I: IntType> From<Character<E>> for Value<'_, E, I> {
	#[inline]
	fn from(character: Character<E>) -> Self {
		Self::Text(Text::from(character))
	}
}

impl<'e, E, I: IntType> From<Variable<'e, E, I>> for Value<'e, E, I> {
	#[inline]
	fn from(variable: Variable<'e, E, I>) -> Self {
		Self::Variable(variable)
	}
}

impl<'e, E, I: IntType> From<Ast<'e, E, I>> for Value<'e, E, I> {
	#[inline]
	fn from(inp: Ast<'e, E, I>) -> Self {
		Self::Ast(inp)
	}
}

impl<'e, E, I: IntType> From<List<'e, E, I>> for Value<'e, E, I> {
	#[inline]
	fn from(list: List<'e, E, I>) -> Self {
		Self::List(list)
	}
}

impl<E, I: IntType> ToBoolean for Value<'_, E, I> {
	fn to_boolean(&self, opts: &Options) -> Result<Boolean> {
		match *self {
			Self::Null => Null.to_boolean(opts),
			Self::Boolean(boolean) => boolean.to_boolean(opts),
			Self::Integer(integer) => integer.to_boolean(opts),
			Self::Text(ref text) => text.to_boolean(opts),
			Self::List(ref list) => list.to_boolean(opts),
			_ => Err(Error::NoConversion { to: Boolean::TYPENAME, from: self.typename() }),
		}
	}
}

impl<E, I: IntType> ToInteger<I> for Value<'_, E, I> {
	fn to_integer(&self, opts: &Options) -> Result<Integer<I>> {
		match *self {
			Self::Null => Null.to_integer(opts),
			Self::Boolean(boolean) => boolean.to_integer(opts),
			Self::Integer(integer) => integer.to_integer(opts),
			Self::Text(ref text) => text.to_integer(opts),
			Self::List(ref list) => list.to_integer(opts),
			_ => Err(Error::NoConversion { to: Integer::<I>::TYPENAME, from: self.typename() }),
		}
	}
}

impl<E: Encoding, I: IntType> ToText<E> for Value<'_, E, I> {
	fn to_text(&self, opts: &Options) -> Result<Text<E>> {
		match *self {
			Self::Null => Null.to_text(opts),
			Self::Boolean(boolean) => boolean.to_text(opts),
			Self::Integer(integer) => integer.to_text(opts),
			Self::Text(ref text) => text.to_text(opts),
			Self::List(ref list) => list.to_text(opts),
			_ => Err(Error::NoConversion { to: Text::<E>::TYPENAME, from: self.typename() }),
		}
	}
}

impl<'e, E, I: IntType> ToList<'e, E, I> for Value<'e, E, I> {
	fn to_list(&self, opts: &Options) -> Result<List<'e, E, I>> {
		match *self {
			Self::Null => Null.to_list(opts),
			Self::Boolean(boolean) => boolean.to_list(opts),
			Self::Integer(integer) => integer.to_list(opts),
			Self::Text(ref text) => text.to_list(opts),
			Self::List(ref list) => list.to_list(opts),
			_ => Err(Error::NoConversion { to: List::<E, I>::TYPENAME, from: self.typename() }),
		}
	}
}

impl<'e, E, I: IntType> Runnable<'e, E, I> for Value<'e, E, I> {
	/// Executes the value.
	fn run(&self, env: &mut Environment<'e, E, I>) -> Result<Self> {
		match self {
			Self::Variable(variable) => variable.run(env),
			Self::Ast(ast) => ast.run(env),
			_ => Ok(self.clone()),
		}
	}
}

impl<'e, E, I: IntType> Value<'e, E, I> {
	/// Fetch the type's name.
	#[must_use = "getting the type name by itself does nothing."]
	pub const fn typename(&self) -> &'static str {
		match self {
			Self::Null => Null::TYPENAME,
			Self::Boolean(_) => Boolean::TYPENAME,
			Self::Integer(_) => Integer::<I>::TYPENAME,
			Self::Text(_) => Text::<E>::TYPENAME,
			Self::List(_) => List::<E, I>::TYPENAME,
			Self::Ast(_) => "Ast",
			Self::Variable(_) => "Variable",
		}
	}
}
