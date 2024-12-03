use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::program::{Compilable, Compiler};
use crate::value::{
	Boolean, Integer, KString, List, NamedType, ToBoolean, ToInteger, ToKString, ToList,
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

impl ToKString for Null {
	/// Simply returns an empty [`KString`].
	#[inline]
	fn to_kstring(&self, _: &mut Environment) -> crate::Result<KString> {
		Ok(KString::default())
	}
}

impl Parseable for Null {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, '_>) -> Result<Option<Self::Output>, ParseError> {
		if parser.advance_if('N').is_none() {
			return Ok(None);
		}

		parser.strip_keyword_function();
		Ok(Some(Self))
	}
}

unsafe impl<'path> Compilable<'path> for Null {
	fn compile(self, compiler: &mut Compiler, _: &Options) -> Result<(), ParseError> {
		compiler.push_constant(self.into());
		Ok(())
	}
}
