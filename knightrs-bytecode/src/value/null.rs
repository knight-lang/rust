use crate::gc::GcRoot;
use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::program::{Compilable, Compiler};
use crate::value::{
	Boolean, Integer, KnString, List, NamedType, ToBoolean, ToInteger, ToKnString, ToList,
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

impl<'gc> ToList<'gc> for Null {
	/// Simply returns an empty [`List`].
	#[inline]
	fn to_list(&self, _: &mut Environment<'gc>) -> crate::Result<GcRoot<'gc, List<'gc>>> {
		Ok(GcRoot::new_unchecked(List::default()))
	}
}

impl<'gc> ToKnString<'gc> for Null {
	/// Simply returns an empty [`KnString`].
	#[inline]
	fn to_knstring(&self, _: &mut Environment<'gc>) -> crate::Result<GcRoot<'gc, KnString<'gc>>> {
		Ok(GcRoot::new_unchecked(KnString::default()))
	}
}

impl<'path> Parseable<'_, 'path, '_> for Null {
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

unsafe impl<'path> Compilable<'_, 'path, '_> for Null {
	fn compile(
		self,
		compiler: &mut Compiler<'_, 'path, '_>,
		_: &Options,
	) -> Result<(), ParseError<'path>> {
		compiler.push_constant(self.into());
		Ok(())
	}
}
