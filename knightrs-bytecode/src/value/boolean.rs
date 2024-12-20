use crate::gc::GcRoot;
use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::program::{Compilable, Compiler};
use crate::strings::KnStr;
use crate::value::{Integer, KnString, List, NamedType, ToInteger, ToKnString, ToList};
use crate::{Environment, Options};

/// The boolean type within Knight.
pub type Boolean = bool;

/// Represents the ability to be converted to a [`Boolean`].
pub trait ToBoolean {
	/// Converts `self` to a [`Boolean`].
	fn to_boolean(&self, env: &mut Environment) -> crate::Result<Boolean>;
}

impl NamedType for Boolean {
	#[inline]
	fn type_name(&self) -> &'static str {
		"Boolean"
	}
}

impl ToBoolean for Boolean {
	/// Simply returns `self`.
	#[inline]
	fn to_boolean(&self, _: &mut Environment) -> crate::Result<Self> {
		Ok(*self)
	}
}

impl ToInteger for Boolean {
	/// Returns `1` for true and `0` for false.
	#[inline]
	fn to_integer(&self, _: &mut Environment) -> crate::Result<Integer> {
		// COMPLIANCE: Both `0` and `1` are always valid integers.
		Ok(Integer::new_unvalidated(*self as i64))
	}
}

impl<'gc> ToList<'gc> for Boolean {
	/// Returns an empty list for `false`, and a list with just `self` if true.
	#[inline]
	fn to_list(&self, _: &mut Environment) -> crate::Result<GcRoot<'gc, List<'gc>>> {
		if *self {
			Ok(GcRoot::new_unchecked(crate::value::list::consts::JUST_TRUE))
		} else {
			Ok(GcRoot::new_unchecked(List::default()))
		}
	}
}

impl<'gc> ToKnString<'gc> for Boolean {
	/// Returns `"true"` for true and `"false"` for false.
	#[inline]
	fn to_knstring(&self, _: &mut Environment<'gc>) -> crate::Result<GcRoot<'gc, KnString<'gc>>> {
		if *self {
			Ok(GcRoot::new_unchecked(crate::value::knstring::consts::TRUE))
		} else {
			Ok(GcRoot::new_unchecked(crate::value::knstring::consts::FALSE))
		}
	}
}

impl<'path> Parseable<'_, 'path> for Boolean {
	type Output = Self;

	fn parse(
		parser: &mut Parser<'_, '_, 'path, '_>,
	) -> Result<Option<Self::Output>, ParseError<'path>> {
		let Some(chr) = parser.advance_if(|c| c == 'T' || c == 'F') else {
			return Ok(None);
		};

		parser.strip_keyword_function();
		Ok(Some(chr == 'T'))
	}
}

unsafe impl<'path> Compilable<'_, 'path> for Boolean {
	fn compile(
		self,
		compiler: &mut Compiler<'_, 'path>,
		_: &Options,
	) -> Result<(), ParseError<'path>> {
		compiler.push_constant(self.into());
		Ok(())
	}
}
