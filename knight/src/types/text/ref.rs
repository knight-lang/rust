use super::{TextInner, Text};
use std::borrow::Borrow;
use std::ops::{Add, Mul, Deref};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TextRef<'a>(pub(super) &'a TextInner);

impl Deref for TextRef<'_> {
	type Target = Text;

	fn deref(&self) -> &Self::Target {
		// SAFETY:
		// `Text` is a transparent pointer to `TextInner` whereas `TextRef` is a transparent
		// reference to the same type. Since pointers and references can be transmuted safely, this is valid.
		unsafe {
			std::mem::transmute::<&TextRef<'_>, &Text>(self)
		}
	}
}

impl Borrow<Text> for TextRef<'_> {
	fn borrow(&self) -> &Text {
		&self
	}
}

impl<T: AsRef<str>> Add<T> for TextRef<'_> {
	type Output = Text;

	fn add(self, rhs: T) -> Self::Output {
		let rhs = rhs.as_ref();

		if rhs.is_empty() {
			return (*self).clone();
		}

		let mut result = String::with_capacity(self.len() + rhs.len());
		result.push_str(self.as_str());
		result.push_str(rhs);

		Text::new(result.into()).unwrap()
	}
}

impl Mul<usize> for TextRef<'_> {
	type Output = Text;

	fn mul(self, amnt: usize) -> Self::Output {
		let mut result = String::with_capacity(self.len() * amnt);

		for _ in 0..amnt {
			result.push_str(self.as_str());
		}

		Text::new(result.into()).unwrap()
	}
}
