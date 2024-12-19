use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::program::{Compilable, Compiler};
use crate::value::{
	Boolean, Integer, KnValueString, List, NamedType, ToBoolean, ToInteger, ToKnValueString, ToList,
};
use crate::{Environment, Options};
use std::fmt::{self, Debug, Formatter};

/// Represents the `NULL` value within Knight.
///
/// Note that this explicitly doesn't implement [`PartialOrd`]/[`Ord`], as you cant compare `NULL`
/// in knight.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Null;

impl Debug for Null {
	#[inline]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str("null")
	}
}

impl NamedType for Null {
	#[inline]
	fn type_name(&self) -> &'static str {
		"Null"
	}
}

impl ToBoolean for Null {
	/// Simply returns `false`.
	#[inline]
	fn to_boolean(&self, _: &mut Environment) -> crate::Result<Boolean> {
		Ok(Boolean::default())
	}
}

impl ToInteger for Null {
	/// Simply returns zero.
	#[inline]
	fn to_integer(&self, _: &mut Environment) -> crate::Result<Integer> {
		Ok(Integer::default())
	}
}

impl ToList for Null {
	/// Simply returns an empty [`List`].
	#[inline]
	fn to_list(&self, _: &mut Environment) -> crate::Result<List> {
		Ok(List::default())
	}
}

impl ToKnValueString for Null {
	/// Simply returns an empty [`KnValueString`].
	#[inline]
	fn to_kstring(&self, _: &mut Environment) -> crate::Result<KnValueString> {
		Ok(KnValueString::default())
	}
}

impl<'path> Parseable<'_, 'path> for Null {
	type Output = Self;

	fn parse(
		parser: &mut Parser<'_, '_, 'path, '_>,
	) -> Result<Option<Self::Output>, ParseError<'path>> {
		if parser.advance_if('N').is_none() {
			return Ok(None);
		}

		parser.strip_keyword_function();
		Ok(Some(Self))
	}
}
/*
unsafe impl<'path> Compilable<'_, 'path> for Null {
	fn compile(
		self,
		compiler: &mut Compiler<'_, 'path>,
		_: &Options,
	) -> Result<(), ParseError<'path>> {
		compiler.push_constant(self.into());
		Ok(())
	}
}
*/
