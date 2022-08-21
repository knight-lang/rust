use crate::knstr::{Chars, IllegalChar, KnStr};
use crate::{Error, Integer};
use std::fmt::{self, Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SharedStr(
	#[cfg(not(feature = "multithreaded"))] std::rc::Rc<KnStr>,
	#[cfg(feature = "multithreaded")] std::sync::Arc<KnStr>,
);

#[cfg(feature = "multithreaded")]
sa::assert_impl_all!(SharedStr: Send, Sync);

impl Default for SharedStr {
	fn default() -> Self {
		SharedStr::new("").unwrap()
	}
}

impl Display for SharedStr {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl std::ops::Deref for SharedStr {
	type Target = KnStr;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<Box<KnStr>> for SharedStr {
	fn from(kstr: Box<KnStr>) -> Self {
		Self(kstr.into())
	}
}
impl TryFrom<String> for SharedStr {
	type Error = IllegalChar;

	fn try_from(inp: String) -> Result<Self, Self::Error> {
		let boxed = Box::<KnStr>::try_from(inp.into_boxed_str())?;

		Ok(Self(boxed.into()))
	}
}

impl SharedStr {
	pub fn new(inp: impl ToString) -> Result<Self, IllegalChar> {
		inp.to_string().try_into()
	}

	pub fn to_integer(&self) -> crate::Result<Integer> {
		let mut bytes = self.trim_start().bytes();

		let (is_negative, mut number) = match bytes.next() {
			Some(b'+') => (false, 0),
			Some(b'-') => (true, 0),
			Some(num @ b'0'..=b'9') => (false, (num - b'0') as Integer),
			_ => return Ok(0),
		};

		while let Some(digit @ b'0'..=b'9') = bytes.next() {
			#[cfg(feature = "checked-overflow")]
			{
				number = number
					.checked_mul(10)
					.and_then(|num| num.checked_add((digit as u8 - b'0') as _))
					.ok_or(Error::IntegerOverflow)?;
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				number = (number * 10) + (digit - b'0') as Integer;
			}
		}

		if is_negative {
			#[cfg(feature = "checked-overflow")]
			{
				number = number.checked_neg().ok_or(Error::IntegerOverflow)?;
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				number = -number;
			}
		}

		Ok(number)
	}
}

impl std::borrow::Borrow<KnStr> for SharedStr {
	fn borrow(&self) -> &KnStr {
		self
	}
}

impl From<&KnStr> for SharedStr {
	fn from(knstr: &KnStr) -> Self {
		Box::<KnStr>::try_from(knstr.to_string().into_boxed_str()).unwrap().into()
	}
}

impl TryFrom<&str> for SharedStr {
	type Error = IllegalChar;

	#[inline]
	fn try_from(inp: &str) -> Result<Self, Self::Error> {
		<&KnStr>::try_from(inp).map(From::from)
	}
}
