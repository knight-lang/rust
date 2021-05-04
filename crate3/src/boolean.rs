use crate::{Number, Text};

pub type Boolean = bool;

impl From<Boolean> for Number {
	#[inline]
	fn from(boolean: Boolean) -> Self {
		const ZERO: Number = unsafe { Number::new_unchecked(0) };
		const ONE: Number = unsafe { Number::new_unchecked(1) };

		if boolean {
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

		if boolean {
			TRUE.text()
		} else {
			FALSE.text()
		}
	}
}
