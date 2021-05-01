use crate::{Number, Text};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Boolean(bool);

impl Boolean {
	pub const fn new(bool: bool) -> Self {
		Self(bool)
	}

	pub const fn inner(self) -> bool {
		self.0
	}
}

impl From<Boolean> for Number {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		const ZERO: Number = unsafe { Number::new_unchecked(0) };
		const ONE: Number = unsafe { Number::new_unchecked(1) };

		if boolean.inner() {
			ONE
		} else {
			ZERO
		}
	}
}

impl From<Boolean> for Text {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		use crate::text::TextStatic;

		const TRUE: TextStatic = unsafe { TextStatic::new_static_unchecked("true") };
		const FALSE: TextStatic = unsafe { TextStatic::new_static_unchecked("false") };

		if boolean.inner() {
			TRUE.text()
		} else {
			FALSE.text()
		}
	}
}

impl From<bool> for Boolean {
	#[inline]
	fn from(bool: bool) -> Self {
		Self::new(bool)
	}
}

impl From<Boolean> for bool {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		boolean.inner()
	}
}

impl AsRef<bool> for Boolean {
	fn as_ref(&self) -> &bool {
		&self.0
	}
}

impl std::borrow::Borrow<bool> for Boolean {
	fn borrow(&self) -> &bool {
		&self.0
	}
}
