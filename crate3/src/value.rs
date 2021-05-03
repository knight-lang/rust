use crate::{Null, Boolean, Text, TextRef, Number, Variable, Ast, Result, Environment, Error};
use std::num::NonZeroU64;
use std::marker::PhantomData;
use std::fmt::{self, Debug, Formatter};

pub struct Value(NonZeroU64);

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

pub trait Runnable : Debug {
	fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Value>;
}

impl Default for Value {
	fn default() -> Self {
		Self::from(Null)
	}
}

impl Value {
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

		Self(NonZeroU64::new_unchecked(data | tag as u64))
	}

	const fn bytes(&self) -> u64 {
		self.0.get()
	}

	const fn unmask(&self) -> u64 {
		self.bytes() & !TAG_MASK
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

impl From<Null> for Value {
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

impl From<Boolean> for Value {
	fn from(bool: Boolean) -> Self {
		if bool.inner() {
			Self::TRUE
		} else {
			Self::FALSE
		}
	}
}

impl From<Text> for Value {
	fn from(text: Text) -> Self {
		unsafe {
			Self::new_tagged(text.into_raw() as usize as u64, Tag::Text)
		}
	}
}

impl From<Number> for Value {
	fn from(number: Number) -> Self {
		unsafe {
			Self::new_tagged((number.inner() as u64) << TAG_BITS, Tag::Text)
		}
	}
}

impl Value {
	pub const fn typename(&self) -> &'static str {
		"todo!()"
	}

	pub const fn as_null(&self) -> Result<Null> {
		if self.bytes() == Self::NULL.0.get() {
			Ok(Null)
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), to: "Null" })
		}
	}

	pub const fn as_boolean(&self) -> Result<Boolean> {
		if self.bytes() == Self::TRUE.bytes() {
			Ok(Boolean::new(true))
		} else if self.bytes() == Self::FALSE.bytes() {
			Ok(Boolean::new(false))
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), to: "Boolean" })
		}
	}

	pub const fn as_number(&self) -> Result<Number> {
		if self.is_tag(Tag::Number) {
			Ok(unsafe { Number::new_unchecked((self.bytes() as i64) >> TAG_BITS) })
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), to: "Number" })
		}
	}

	pub fn as_text(&self) -> Result<TextRef> {
		if self.is_tag(Tag::Text) {
			unsafe {
				Ok(TextRef::from_raw(self.unmask() as _))
			}
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), to: "Text" })
		}
	}

	pub fn as_variable(&self) -> Result<Variable> {
		if self.is_tag(Tag::Variable) {
			unsafe {
				Ok(Variable::from_raw(self.unmask() as _))
			}
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), to: "Variable" })
		}
	}

	pub fn as_ast(&self) -> Result<Ast> {
		if self.is_tag(Tag::Ast) {
			unsafe {
				Ok(Ast::from_raw(self.unmask() as _))
			}
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), to: "Ast" })
		}
	}

	pub fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Self> {
		let _ = env;
		todo!()
	}
}


impl Clone for Value {
	fn clone(&self) -> Self {
		todo!()
	}
}

// impl Drop for Value {
// 	fn drop(&mut self) {
// 		// todo
// 	}
// }

impl Debug for Value {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		todo!()
	}
}
