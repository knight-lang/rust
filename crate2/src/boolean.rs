use crate::{Value, value::Idempotent, Number, Text};
use std::fmt::{self, Display, Formatter};
// use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Boolean(bool);

// const FALSE_BYTES: u64 = 0;
// const TRUE_BYTES: u64 = 1 << 4;

impl Boolean {
	pub const fn new(boolean: bool) -> Self {
		Self(boolean)
	}

	pub const fn is_true(self) -> bool {
		self.0
	}
}

impl From<Boolean> for Value {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		// sa::const_assert_eq!(FALSE_BYTES, (false as u64) << 4);
		// sa::const_assert_eq!(TRUE_BYTES, (true as u64) << 4);

		unsafe {
			Self::from_bytes((boolean.0 as u64) << 4)
		}
	}
}

impl Display for Boolean {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.0, f)
	}
}

impl From<bool> for Boolean {
	fn from(boolean: bool) -> Self {
		Self::new(boolean)
	}
}

impl From<Boolean> for bool {
	fn from(boolean: Boolean) -> Self {
		boolean.0
	}
}

impl AsRef<bool> for Boolean {
	fn as_ref(&self) -> &bool {
		&self.0
	}
}
impl From<Boolean> for Number {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		unsafe {
			Self::new_unchecked((boolean.0 as i64) >> 4)
		}
	}
}

impl From<Boolean> for Text {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		use crate::text::TextStatic;

		static TRUE_TEXT: TextStatic = unsafe { TextStatic::new_unchecked("true") };
		static FALSE_TEXT: TextStatic = unsafe { TextStatic::new_unchecked("false") };

		if boolean.0 {
			TRUE_TEXT.as_text()
		} else {
			FALSE_TEXT.as_text()
		}
	}
}

impl Idempotent for Boolean {}

#[cfg(test)]
#[allow(unused)]
mod tests {
	use super::*;
	use std::convert::TryFrom;
	use crate::value::ValueKind;

	// #[test]
	// fn to_boolean() {
	// 	assert_eq!(Boolean::new(true).to_boolean().unwrap(), Boolean::new(true));
	// 	assert_eq!(Boolean::new(false).to_boolean().unwrap(), Boolean::new(false));
	// }

	// #[test]
	// fn to_number() {
	// 	assert_eq!(Boolean::new(true).to_number().unwrap(), Number::new(1).unwrap());
	// 	assert_eq!(Boolean::new(false).to_number().unwrap(), Number::new(0).unwrap());
	// }

	// #[test]
	// fn to_text() {
	// 	assert_eq!(Boolean::new(true).to_text().unwrap(), Text::try_from("true").unwrap());
	// 	assert_eq!(Boolean::new(false).to_text().unwrap(), Text::try_from("false").unwrap());
	// }

	#[test]
	fn to_value_and_back() {
		// assert_eq!(Boolean::new(true), Boolean::try_from(Value::from(Boolean::new(true))).unwrap());
	}
}