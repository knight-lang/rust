use crate::{Value, value::Idempotent, Number, Text, Boolean};
use std::fmt::{self, Display, Formatter};

// no `PartialOrd` or `Ord` deives because in Knight, Null isn't comparable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Null;

impl Display for Null {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt("null", f)
	}
}

impl From<()> for Null {
	#[inline]
	fn from(_: ()) -> Self {
		Self
	}
}

impl Null {
	pub const fn into_value_const(self) -> Value {
		unsafe {
			Value::from_bytes(8) // 8 is null's byte repr
		}
	}
}

impl From<Null> for Value {
	#[inline]
	fn from(_: Null) -> Self {
		Null.into_value_const()
	}
}

impl From<Null> for Boolean {
	#[inline]
	fn from(_: Null) -> Self {
		Self::new(false)
	}
}

impl From<Null> for Number {
	#[inline]
	fn from(_: Null) -> Self {
		unsafe {
			Self::new_unchecked(0)
		}
	}
}

impl From<Null> for Text {
	#[inline]
	fn from(_: Null) -> Self {
		use crate::text::TextStatic;

		static NULL_TEXT: TextStatic = unsafe { TextStatic::new_unchecked("null") };

		NULL_TEXT.as_text()
	}
}

impl Idempotent for Null {}
