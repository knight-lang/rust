use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::program::{Compilable, Compiler};
use crate::strings::StringSlice;
use crate::value::{Integer, KString, List, NamedType, ToInteger, ToKString, ToList};
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

impl ToList for Boolean {
	/// Returns an empty list for `false`, and a list with just `self` if true.
	#[inline]
	fn to_list(&self, _: &mut Environment) -> crate::Result<List> {
		// static TRUE_BOX: List = List::new_unvalidated(true.into());
		if *self {
			Ok(List::boxed((*self).into()))
		} else {
			Ok(List::default())
		}
	}
}

impl ToKString for Boolean {
	/// Returns `"true"` for true and `"false"` for false.
	#[inline]
	fn to_kstring(&self, _: &mut Environment) -> crate::Result<KString> {
		// VALIDATION: `true` and `false` are always valid strings.
		static TRUE: &StringSlice = StringSlice::new_unvalidated("true");
		static FALSE: &StringSlice = StringSlice::new_unvalidated("false");

		// TODO: make sure this isn't allocating each time
		if *self {
			Ok(TRUE.into())
		} else {
			Ok(FALSE.into())
		}
	}
}

impl<'path> Parseable<'_, 'path> for Boolean {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, 'path>) -> Result<Option<Self::Output>, ParseError<'path>> {
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
