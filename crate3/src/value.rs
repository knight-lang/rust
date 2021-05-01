use crate::{Null, Boolean, Text, Number, Variable, Ast};
use std::num::NonZeroU64;
use std::marker::PhantomData;

pub struct Value<'env>(NonZeroU64, PhantomData<&'env ()>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tag {
	Constant = 0,
	Number,
	Variable,
	Text,
	Ast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Constant {
	Undefined = 0,
	True,
	False,
	Null,
}

pub(crate) const TAG_BITS: u64 = 3;
const TAG_MASK: u64 = (1 << TAG_BITS) - 1;

impl Default for Value<'_> {
	fn default() -> Self {
		Self::from(Null)
	}
}

impl Value<'_> {
	const NULL: Self = unsafe {
		Self::new_tagged((Constant::Null as u64) << TAG_BITS, Tag::Constant)
	};

	const TRUE: Self = unsafe {
		Self::new_tagged((Constant::True as u64) << TAG_BITS, Tag::Constant)
	};

	const FALSE: Self = unsafe {
		Self::new_tagged((Constant::False as u64) << TAG_BITS, Tag::Constant)
	};

	const unsafe fn new_tagged(data: u64, tag: Tag) -> Self {
		debug_assert_eq_const!(data & TAG_MASK, 0, "invalid bits given");
		debug_assert_ne_const!(data | tag as u64, 0, "undefined value created");

		Self(NonZeroU64::new_unchecked(data | tag as u64), PhantomData)
	}

	const fn bytes(&self) -> u64 {
		self.0.get()
	}

	const fn tag(&self) -> Tag {
		match self.bytes() & TAG_BITS {
			0 => Tag::Constant,
			1 => Tag::Number,
			2 => Tag::Variable,
			3 => Tag::Text,
			4 => Tag::Ast,
			other => {
				debug_assert_eq_const!(other, 0xff, "nope");
				Tag::Ast // todo: how do we handle UB?
			}
		}
	}

	const fn is_tag(&self, tag: Tag) -> bool {
		self.tag() as u64 == tag as u64
	}

	pub fn new<T: Into<Self>>(data: T) -> Self {
		data.into()
	}
}

impl From<Null> for Value<'_> {
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

impl From<Boolean> for Value<'_> {
	fn from(bool: Boolean) -> Self {
		if bool.inner() {
			Self::TRUE
		} else {
			Self::FALSE
		}
	}
}

impl From<Text> for Value<'_> {
	fn from(text: Text) -> Self {
		unsafe {
			Self::new_tagged(text.into_raw().get() as u64, Tag::Text)
		}
	}
}

impl From<Number> for Value<'_> {
	fn from(number: Number) -> Self {
		unsafe {
			Self::new_tagged((number.inner() as u64) << TAG_BITS, Tag::Text)
		}
	}
}

impl<'env> Value<'env> {
	pub const fn as_null(&self) -> Option<Null> {
		if self.bytes() == Self::NULL.bytes() {
			Some(Null)
		} else {
			None
		}
	}

	pub const fn as_boolean(&self) -> Option<Boolean> {
		if self.bytes() == Self::TRUE.bytes() {
			Some(Boolean::new(true))
		} else if self.bytes() == Self::FALSE.bytes() {
			Some(Boolean::new(false))
		} else {
			None
		}
	}

	pub const fn as_number(&self) -> Option<Number> {
		if self.is_tag(Tag::Number) {
			Some(unsafe { Number::new_unchecked((self.bytes() as i64) >> TAG_BITS) })
		} else {
			None
		}
	}

	pub fn as_text(&self) -> Option<&Text> {
		if self.is_tag(Tag::Text) {
			unsafe {
				Some(&Text::from_raw_ref(&self.0))
			}
		} else {
			None
		}
	}

	pub fn as_variable(&self) -> Option<Variable<'env>> {
		if self.is_tag(Tag::Variable) {
			unsafe {
				Some(Variable::from_raw(self.0))
			}
		} else {
			None
		}
	}

	pub fn ast_ast(&self) -> Option<Ast<'env>> {
		if self.is_tag(Tag::Ast) {
			unsafe {
				Some(Ast::from_raw(self.0))
			}
		} else {
			None
		}
	}
}