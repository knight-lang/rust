use crate::container::RefCount;
use crate::parser::{ParseError, ParseErrorKind, Parseable, Parser};
use crate::program::{Compilable, Compiler};
use crate::strings::{KnStr, StringError};
use crate::value::{Boolean, Integer, List, NamedType, ToBoolean, ToInteger, ToList};
use crate::{Environment, Options};
use std::borrow::{Borrow, Cow};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // TODO, debug
#[repr(align(16))] // DELETEME, used when testing `value2`
pub struct KnValueString(RefCount<KnStr>);

pub trait ToKnValueString {
	fn to_kstring(&self, env: &mut Environment) -> crate::Result<KnValueString>;
}

impl NamedType for KnValueString {
	#[inline]
	fn type_name(&self) -> &'static str {
		"String"
	}
}

impl Default for KnValueString {
	fn default() -> Self {
		Self::from_slice(Default::default())
	}
}

impl From<&KnStr> for KnValueString {
	fn from(slice: &KnStr) -> Self {
		Self::from_slice(slice)
	}
}

impl KnValueString {
	pub fn from_slice(slice: &KnStr) -> Self {
		let refcounted = RefCount::<str>::from(slice.as_str());
		// SAFETY: tood, but it is valid i think lol
		Self(unsafe { RefCount::from_raw(RefCount::into_raw(refcounted) as *const KnStr) })
	}

	pub fn from_string_unchecked(source: String) -> Self {
		let refcounted = RefCount::<str>::from(source);
		// SAFETY: tood, but it is valid i think lol
		Self(unsafe { RefCount::from_raw(RefCount::into_raw(refcounted) as *const KnStr) })
	}

	/// Creates a new `KnValueString` without validating it.
	///
	/// # Validation
	/// The `source` must only contain bytes valid in all encodings, and must be less than the max
	/// length for containers.
	pub fn new_unvalidated(source: String) -> Self {
		Self::from_slice(KnStr::new_unvalidated(&source))
	}

	#[cfg_attr(not(feature = "compliance"), inline)]
	pub fn new(source: String, opts: &Options) -> Result<Self, crate::strings::StringError> {
		KnStr::new(&source, opts).map(Self::from_slice)
	}
}

impl Borrow<KnStr> for KnValueString {
	fn borrow(&self) -> &KnStr {
		&self
	}
}

impl std::ops::Deref for KnValueString {
	type Target = KnStr;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl AsRef<KnStr> for KnValueString {
	fn as_ref(&self) -> &KnStr {
		&self
	}
}

impl ToBoolean for KnValueString {
	fn to_boolean(&self, _: &mut Environment) -> crate::Result<Boolean> {
		Ok(!self.is_empty())
	}
}

impl ToKnValueString for KnValueString {
	fn to_kstring(&self, _: &mut Environment) -> crate::Result<KnValueString> {
		Ok(self.clone())
	}
}

impl ToInteger for KnValueString {
	fn to_integer(&self, env: &mut Environment) -> crate::Result<Integer> {
		Integer::parse_from_str(self.as_str(), env.opts())
	}
}

impl ToList for KnValueString {
	fn to_list(&self, env: &mut Environment) -> crate::Result<List> {
		let chars =
			self.chars().map(|c| Self::new_unvalidated(c.to_string()).into()).collect::<Vec<_>>();

		// COMPLIANCE: If `self` is within the container bounds, so is the length of its chars.
		Ok(List::new_unvalidated(chars))
	}
}

impl KnValueString {
	/// Concatenates two strings together
	pub fn concat(&self, rhs: &KnStr, opts: &Options) -> Result<Self, StringError> {
		if self.is_empty() {
			return Ok(rhs.to_owned());
		}

		if rhs.is_empty() {
			return Ok(self.clone());
		}

		let str = self.as_str().to_owned() + rhs.as_str();
		Self::new(str, opts)
	}

	pub fn remove_substr(&self, substr: &KnStr) -> Self {
		let _ = substr;
		todo!();
	}

	/// Gets the first character of `self`, if it exists.
	pub fn head(&self) -> Option<char> {
		self.chars().next()
	}

	/// Gets everything _but_ the first character of `self`, if it exists.
	pub fn tail(&self) -> Option<Self> {
		self.0.get(1..).map(Self::from)
	}

	pub fn repeat(&self, amount: usize, opts: &Options) -> Result<KnValueString, StringError> {
		// Make sure `str.repeat()` won't panic
		if amount.checked_mul(self.len()).map_or(true, |c| isize::MAX as usize <= c) {
			// TODO: maybe we don't have the length in `LengthTooLong` ?
			// return Err(StringError::LengthTooLong(self.len().wrapping_mul(amount)));
			todo!();
		}

		#[cfg(feature = "compliance")]
		if opts.compliance.check_container_length && KnStr::COMPLIANCE_MAX_LEN < self.len() * amount {
			return Err(StringError::LengthTooLong(self.len() * amount));
		}

		Ok(KnValueString::from_string_unchecked(self.as_str().repeat(amount)))
	}

	#[cfg(feature = "extensions")]
	#[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	pub fn split(&self, sep: &Self, env: &mut Environment) -> List {
		if sep.is_empty() {
			// TODO: optimize me
			return crate::Value::from(self.to_owned()).to_list(env).unwrap();
		}

		// SAFETY: If `self` is within the container bounds, so is the length of its chars.
		List::new_unvalidated(
			self
				.as_str()
				.split(sep.as_str())
				.map(|s| KnValueString::new_unvalidated(s.to_string()))
				.map(crate::Value::from),
		)
	}

	pub fn ord(&self, opts: &Options) -> crate::Result<Integer> {
		let chr = self.chars().next().ok_or(crate::Error::DomainError("empty string"))?;
		// technically not redundant in case checking for ints is enabled but not strings.
		Integer::new_error(u32::from(chr) as _, opts).map_err(From::from)
	}
}

impl<'path> Parseable<'_, 'path> for KnValueString {
	type Output = Self;

	fn parse(parser: &mut Parser<'_, '_, 'path>) -> Result<Option<Self::Output>, ParseError<'path>> {
		#[cfg(feature = "extensions")]
		if parser.opts().extensions.syntax.string_interpolation && parser.advance_if('`').is_some() {
			todo!();
		}

		let Some(quote) = parser.advance_if(|c| c == '\'' || c == '\"') else {
			return Ok(None);
		};

		let start = parser.location();

		// empty stings are allowed to exist
		let contents = parser.take_while(|c| c != quote).unwrap_or_default();

		if parser.advance_if(quote).is_none() {
			return Err(start.error(ParseErrorKind::MissingEndingQuote(quote)));
		}

		let string = KnValueString::new(contents.to_string(), parser.opts())
			.map_err(|err| start.error(err.into()))?;
		Ok(Some(string))
	}
}

unsafe impl<'path> Compilable<'_, 'path> for KnValueString {
	fn compile(
		self,
		compiler: &mut Compiler<'_, 'path>,
		_: &Options,
	) -> Result<(), ParseError<'path>> {
		compiler.push_constant(self.into());
		Ok(())
	}
}

impl KnValueString {
	// /// Gets an iterate over [`Character`]s.
	// pub fn chars(&self) -> Chars<'_> {
	// 	Chars(self.0.chars())
	// }

	// pub fn get<T: std::slice::SliceIndex<str, Output = str>>(&self, range: T) -> Option<&Self> {
	// 	let substring = self.0.get(range)?;

	// 	// SAFETY: We're getting a substring of a valid TextSlice, which thus will itself be valid.
	// 	Some(unsafe { Self::new_unchecked(substring) })
	// }

	// /// Concatenates two strings together
	// pub fn concat(&self, rhs: &Self, opts: &Options) -> Result<KnValueString, NewTextError> {
	// 	let mut builder = super::Builder::with_capacity(self.len() + rhs.len());

	// 	builder.push(self);
	// 	builder.push(rhs);

	// 	builder.finish(flags)
	// }

	// pub fn repeat(&self, amount: usize, opts: &Options) -> Result<KnValueString, NewTextError> {
	// 	unsafe { KnValueString::new_len_unchecked((**self).repeat(amount), flags) }
	// }

	// #[cfg(feature = "extensions")]
	// #[cfg_attr(docsrs, doc(cfg(feature = "extensions")))]
	// pub fn split(&self, sep: &Self, env: &mut Environment) -> List {
	// 	if sep.is_empty() {
	// 		// TODO: optimize me
	// 		return Value::from(self.to_owned()).to_list(env).unwrap();
	// 	}

	// 	let chars = (**self)
	// 		.split(&**sep)
	// 		.map(|x| unsafe { KnValueString::new_unchecked(x) }.into())
	// 		.collect::<Vec<_>>();

	// 	// SAFETY: If `self` is within the container bounds, so is the length of its chars.
	// 	unsafe { List::new_unchecked(chars) }
	// }

	// pub fn ord(&self) -> crate::Result<Integer> {
	// 	Integer::try_from(self.chars().next().ok_or(crate::Error::DomainError("empty string"))?)
	// }

	// /// Gets the first character of `self`, if it exists.
	// pub fn head(&self) -> Option<char> {
	// 	self.chars().next()
	// }

	// /// Gets everything _but_ the first character of `self`, if it exists.
	// pub fn tail(&self) -> Option<&TextSlice> {
	// 	self.get(1..)
	// }

	// pub fn remove_substr(&self, substr: &Self) -> KnValueString {
	// 	let _ = substr;
	// 	todo!();
	// }
}
