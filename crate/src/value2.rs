use crate::{Text, Variable, Ast, number::{Number, NumberType}};
use std::num::NonZeroU64;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::mem::ManuallyDrop;

pub struct Value(NonZeroU64);

sa::const_assert!(std::mem::size_of::<Number>() <= std::mem::size_of::<Value>());
sa::const_assert!(std::mem::size_of::<NumberType>() <= std::mem::size_of::<Value>());
sa::const_assert!(std::mem::size_of::<*const ()>() <= std::mem::size_of::<Value>());

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tag {
	Null     = 0b001,
	Boolean  = 0b010,
	Number   = 0b011,
	Variable = 0b100,
	Text     = 0b101,
	Ast      = 0b110,
	#[cfg(feature="custom-types")]
	Custom   = 0b111,
}

pub(crate) const TAG_SHIFT: u64 = 3;
const TAG_MASK: u64 = (1<<TAG_SHIFT)-1;

impl Value {
	pub const NULL: Self = unsafe { Self::new_tagged(0, Tag::Null) };
	pub const TRUE: Self = unsafe { Self::new_tagged(1 << TAG_SHIFT, Tag::Boolean) };
	pub const FALSE: Self = unsafe { Self::new_tagged(0, Tag::Boolean) };
	pub const ZERO: Self = unsafe { Self::new_number_unchecked(1) };

	const fn bytes(&self) -> u64 {
		self.0.get()
	}

	const fn tag(&self) -> Tag {
		match self.bytes() & TAG_MASK {
			0b001 => Tag::Null,
			0b010 => Tag::Boolean,
			0b011 => Tag::Number,
			0b100 => Tag::Text,
			0b101 => Tag::Variable,
			0b110 => Tag::Ast,
			#[cfg(feature="custom-types")]
			0b111 => Tag::Custom,
			_ => {
				let _: () = [/* unreachable */][self.bytes() as usize];
				loop{}
			}
		}
	}

	const fn is_tag(&self, tag: Tag) -> bool {
		(self.bytes() & TAG_MASK) == (tag as u64)
	}

	const unsafe fn new_tagged(data: u64, tag: Tag) -> Self {
		let inner = data | (tag as u64);

		debug_assert_ne_const!(data & TAG_MASK, 0);
		debug_assert_ne_const!(inner, 0);

		Self(NonZeroU64::new_unchecked(inner))
	}

	#[inline]
	pub const fn is_null(&self) -> bool {
		self.is_tag(Tag::Null)
	}

	#[inline]
	pub const fn new_boolean(boolean: bool) -> Self {
		if boolean {
			Self::TRUE
		} else {
			Self::FALSE
		}
	}

	#[inline]
	pub const fn is_boolean(&self) -> bool {
		self.is_tag(Tag::Boolean)
	}

	#[inline]
	pub const fn as_boolean(&self) -> Option<bool> {
		if self.is_boolean() {
			Some(unsafe { self.as_boolean_unchecked() })
		} else {
			None
		}
	}

	#[inline]
	pub const unsafe fn as_boolean_unchecked(&self) -> bool {
		debug_assert_const!(self.is_boolean());

		self.bytes() != (Tag::Boolean as u64)
	}

	pub const fn new_number(num: NumberType) -> Option<Self> {
		if Number::is_valid(num) {
			Some(unsafe { Self::new_number_unchecked(num) })
		} else {
			None
		}
	}

	#[inline]
	pub const unsafe fn new_number_unchecked(num: NumberType) -> Self {
		debug_assert_const!(Number::is_valid(num));

		Self::new_tagged((num as u64) << TAG_SHIFT, Tag::Number)
	}

	#[inline]
	pub const fn is_number(&self) -> bool {
		self.is_tag(Tag::Number)
	}

	#[inline]
	pub const fn as_number(&self) -> Option<Number> {
		if self.is_number() {
			Some(unsafe { self.as_number_unchecked() })
		} else {
			None
		}
	}

	#[inline]
	pub const unsafe fn as_number_unchecked(&self) -> Number {
		debug_assert_const!(self.is_number());

		Number::new_unchecked((self.bytes() as NumberType) >> TAG_SHIFT)
	}

	#[inline]
	pub const fn is_literal(&self) -> bool {
		(self.tag() as u64) >= (Tag::Variable as u64)
	}

	#[inline(always)]
	const unsafe fn copy(&self) -> Self {
		Self(NonZeroU64::new_unchecked(self.bytes()))
	}

	#[inline]
	pub fn new_text(text: Text) -> Self {
		Self(text.into_raw())
	}

	#[inline]
	pub fn is_text(&self) -> bool {
		self.is_tag(Tag::Text)
	}

	#[inline]
	pub fn into_text(self) -> Result<Text, Self> {
		if self.is_text() {
			Ok(unsafe { self.into_text_unchecked() })
		} else {
			Err(self)
		}
	}

	pub unsafe fn into_text_unchecked(self) -> Text {
		debug_assert_const!(self.is_text());
		Text::from_raw(self.0)
	}

	#[inline]
	pub fn as_text(&self) -> Option<Text> {
		if self.is_text() {
			Some(unsafe { self.as_text_unchecked() })
		} else {
			None
		}
	}

	pub unsafe fn as_text_unchecked(&self) -> Text {
		debug_assert_const!(self.is_text());
		let text = ManuallyDrop::new(Text::from_raw(self.0));
		(*text).clone()
	}

	#[inline]
	pub fn new_variable(variable: Variable) -> Self {
		Self(variable.into_raw())
	}

	#[inline]
	pub fn is_variable(&self) -> bool {
		self.is_tag(Tag::Variable)
	}

	#[inline]
	pub fn into_variable(self) -> Result<Variable, Self> {
		if self.is_variable() {
			Ok(unsafe { self.into_variable_unchecked() })
		} else {
			Err(self)
		}
	}

	pub unsafe fn into_variable_unchecked(self) -> Variable {
		debug_assert_const!(self.is_variable());
		Variable::from_raw(self.0)
	}

	#[inline]
	pub fn as_variable(&self) -> Option<Variable> {
		if self.is_variable() {
			Some(unsafe { self.as_variable_unchecked() })
		} else {
			None
		}
	}

	pub unsafe fn as_variable_unchecked(&self) -> Variable {
		debug_assert_const!(self.is_variable());

		// we need to clone it, as we'd be taking an owned reference otherwise.
		let variable = std::mem::ManuallyDrop::new(Variable::from_raw(self.0));
		(*variable).clone()
	}


	#[inline]
	pub fn new_ast(ast: Ast) -> Self {
		Self(ast.into_raw())
	}

	#[inline]
	pub fn is_ast(&self) -> bool {
		self.is_tag(Tag::Ast)
	}

	#[inline]
	pub fn into_ast(self) -> Result<Ast, Self> {
		if self.is_ast() {
			Ok(unsafe { self.into_ast_unchecked() })
		} else {
			Err(self)
		}
	}

	pub unsafe fn into_ast_unchecked(self) -> Ast {
		debug_assert_const!(self.is_ast());
		Ast::from_raw(self.0)
	}

	#[inline]
	pub fn as_ast(&self) -> Option<Ast> {
		if self.is_ast() {
			Some(unsafe { self.as_ast_unchecked() })
		} else {
			None
		}
	}

	pub unsafe fn as_ast_unchecked(&self) -> Ast {
		debug_assert_const!(self.is_ast());

		// we need to clone it, as we'd be taking an owned reference otherwise.
		let ast = std::mem::ManuallyDrop::new(Ast::from_raw(self.0));
		(*ast).clone()
	}
}

impl Clone for Value {
	fn clone(&self) -> Self {
		use std::mem::ManuallyDrop;

		unsafe {
			//increase refcount for all these types, then just copy us.
			match self.tag() {
				Tag::Null | Tag::Boolean | Tag::Number => { /* do nothing */ },
				Tag::Variable => drop(ManuallyDrop::new(self.as_variable_unchecked())),
				Tag::Text => drop(ManuallyDrop::new(self.as_text_unchecked())),
				Tag::Ast => drop(ManuallyDrop::new(self.as_ast_unchecked())),
				#[cfg(feature="custom-types")]
				Tag::Custom => todo!()
			}

			self.copy()
		}
	}
}

impl Drop for Value {
	fn drop(&mut self) {
		unsafe {
			// we have to drop it twice, once for the as-ref and once for 
			match self.tag() {
				Tag::Null | Tag::Boolean | Tag::Number => { /* do nothing */ },
				Tag::Variable => drop(Variable::from_raw(self.0)),
				Tag::Text => drop(Text::from_raw(self.0)),
				Tag::Ast => drop(Ast::from_raw(self.0)),
				#[cfg(feature="custom-types")]
				Tag::Custom => todo!()
			}
		}
	}
}

impl Default for Value {
	fn default() -> Self {
		unsafe {
			Self::NULL.copy()
		}
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self.tag() {
			Tag::Null => f.debug_tuple("Null").finish(),
			Tag::Boolean => f.debug_tuple("Boolean").field(&self.as_boolean().unwrap()).finish(),
			Tag::Number => Debug::fmt(&self.as_number().unwrap(), f),
			Tag::Text => Debug::fmt(&self.as_text().unwrap(), f),
			Tag::Variable => Debug::fmt(&self.as_variable().unwrap(), f),
			Tag::Ast => Debug::fmt(&self.as_ast().unwrap(), f),
			#[cfg(feature="custom-types")]
			Tag::Custom => todo!()
		}
	}
}