use crate::{Variable, Ast, Result, Error, Environment, Null};
use crate::text::{ToText, Text, TextRef, TextCow};
use crate::number::{ToNumber, Number, NumberType};
use crate::boolean::{ToBoolean, Boolean};

use std::num::NonZeroU64;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::mem::ManuallyDrop;
use std::convert::TryFrom;
use std::cmp::Ordering;
use std::borrow::Cow;
use crate::ops::*;

pub struct Value(NonZeroU64);

const_assert!(std::mem::size_of::<Number>() <= std::mem::size_of::<Value>());
const_assert!(std::mem::size_of::<NumberType>() <= std::mem::size_of::<Value>());
const_assert!(std::mem::size_of::<*const ()>() <= std::mem::size_of::<Value>());

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

#[derive(Debug)]
pub enum ValueKind<'a> {
	Null,
	Boolean(Boolean),
	Number(Number),
	Variable(Variable),
	Text(TextRef<'a>),
	Ast(Ast),
	#[cfg(feature="custom-types")]
	Custom(())
}

pub(crate) const TAG_SHIFT: u64 = 3;
const TAG_MASK: u64 = (1<<TAG_SHIFT)-1;

impl Value {
	pub const NULL: Self = unsafe { Self::new_tagged(0, Tag::Null) };
	pub const TRUE: Self = unsafe { Self::new_tagged(1 << TAG_SHIFT, Tag::Boolean) };
	pub const FALSE: Self = unsafe { Self::new_tagged(0, Tag::Boolean) };
	pub const ZERO: Self = unsafe { Self::new_num_unchecked(0) };

	const fn bytes(&self) -> u64 {
		self.0.get()
	}

	const fn unmask(&self) -> u64 {
		self.bytes() & !TAG_MASK
	}

	const fn ptr(&self) -> *const () {
		self.unmask() as *const ()
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
	pub const fn new_null() -> Self {
		unsafe {
			Self::new_tagged(0, Tag::Null)
		}
	}

	#[inline]
	pub const fn is_null(&self) -> bool {
		self.is_tag(Tag::Null)
	}

	#[inline]
	pub const fn new_boolean(boolean: bool) -> Self {
		unsafe {
			Self::new_tagged((boolean as u64) << TAG_SHIFT, Tag::Boolean)
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

	pub const fn new_num(num: NumberType) -> Option<Self> {
		// `Option::map` isn't a constfn.
		if let Some(number) = Number::new(num) {
			Some(Self::new_number(number))
		} else {
			None
		}
	}

	#[inline]
	pub const unsafe fn new_num_unchecked(num: NumberType) -> Self {
		Self::new_number(Number::new_unchecked(num))
	}


	pub const fn new_number(num: Number) -> Self {
		unsafe {
			Self::new_tagged((num.get() as u64) << TAG_SHIFT, Tag::Number)
		}
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

	pub fn new_str(str: &str) -> std::result::Result<Self, crate::text::InvalidChar> {
		Text::new(str).map(Self::new_text)
	}

	#[inline]
	pub fn new_text(text: Text) -> Self {
		unsafe {
			Self::new_tagged(text.into_raw() as u64, Tag::Text)
		}
	}

	#[inline]
	pub const fn is_text(&self) -> bool {
		self.is_tag(Tag::Text)
	}

	#[inline]
	pub fn into_text(self) -> std::result::Result<Text, Self> {
		if self.is_text() {
			Ok(unsafe { self.into_text_unchecked() })
		} else {
			Err(self)
		}
	}

	pub unsafe fn into_text_unchecked(self) -> Text {
		debug_assert_const!(self.is_text());
		Text::from_raw(self.ptr())
	}

	#[inline]
	pub fn as_text(&self) -> Option<TextRef<'_>> {
		if self.is_text() {
			Some(unsafe { self.as_text_unchecked() })
		} else {
			None
		}
	}

	pub unsafe fn as_text_unchecked(&self) -> TextRef<'_> {
		debug_assert_const!(self.is_text());

		TextRef::from_raw(self.ptr())
	}

	#[inline]
	pub fn new_variable(variable: Variable) -> Self {
		unsafe {
			Self::new_tagged(variable.into_raw() as u64, Tag::Variable)
		}
	}

	#[inline]
	pub fn is_variable(&self) -> bool {
		self.is_tag(Tag::Variable)
	}

	#[inline]
	pub fn into_variable(self) -> std::result::Result<Variable, Self> {
		if self.is_variable() {
			Ok(unsafe { self.into_variable_unchecked() })
		} else {
			Err(self)
		}
	}

	pub unsafe fn into_variable_unchecked(self) -> Variable {
		debug_assert_const!(self.is_variable());
		Variable::from_raw(self.ptr())
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
		let variable = std::mem::ManuallyDrop::new(Variable::from_raw(self.ptr()));
		(*variable).clone()
	}


	#[inline]
	pub fn new_ast(ast: Ast) -> Self {
		unsafe {
			Self::new_tagged(ast.into_raw() as u64, Tag::Ast)
		}
	}

	#[inline]
	pub fn is_ast(&self) -> bool {
		self.is_tag(Tag::Ast)
	}

	#[inline]
	pub fn into_ast(self) -> std::result::Result<Ast, Self> {
		if self.is_ast() {
			Ok(unsafe { self.into_ast_unchecked() })
		} else {
			Err(self)
		}
	}

	pub unsafe fn into_ast_unchecked(self) -> Ast {
		debug_assert_const!(self.is_ast());
		Ast::from_raw(self.ptr())
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
		let ast = std::mem::ManuallyDrop::new(Ast::from_raw(self.ptr()));
		(*ast).clone()
	}


	pub fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Self> {
		match self.tag() {
			Tag::Null
				| Tag::Boolean
				| Tag::Number => Ok(unsafe { self.copy() }),

			Tag::Text => unsafe {
				Text::clone_in_place(self.ptr());
				Ok(self.copy())
			},

			Tag::Variable => {
				let variable = unsafe { self.as_variable_unchecked() };

				variable
					.fetch()
					.ok_or_else(|| Error::UnknownIdentifier { identifier: variable.name().into() })
			},

			Tag::Ast => unsafe { self.as_ast_unchecked() }.run(env),

			#[cfg(feature="custom-types")]
			Tag::Custom => todo!()
		}
	}

	pub fn typename(&self) -> &'static str {
		match self.tag() {
			Tag::Null     => "Null",
			Tag::Boolean  => "Boolean",
			Tag::Number   => "Number",
			Tag::Text     => "Text",
			Tag::Variable => "Variable",
			Tag::Ast      => "Ast",
			#[cfg(feature="custom-types")]
			Tag::Custom   => todo!("dereference custom types"),
		}
	}

	pub fn classify(&self) -> ValueKind<'_> {
		unsafe {
			match self.tag() {
				Tag::Null => ValueKind::Null,
				Tag::Boolean => ValueKind::Boolean(self.as_boolean_unchecked()),
				Tag::Number => ValueKind::Number(self.as_number_unchecked()),
				Tag::Text => ValueKind::Text(self.as_text_unchecked()),
				Tag::Variable => todo!(),
				Tag::Ast => todo!(),
				#[cfg(feature="custom-types")]
				Tag::Custom => todo!()
			}
		}
	}
}

impl ToBoolean for Value {
	fn to_boolean(&self) -> crate::Result<Boolean> {
		unsafe {
			match self.tag() {
				Tag::Null => Null.to_boolean(),
				Tag::Boolean => self.as_boolean_unchecked().to_boolean(),
				Tag::Number => self.as_number_unchecked().to_boolean(),
				Tag::Text => self.as_text_unchecked().to_boolean(),
				_ => Err(Error::UndefinedConversion { into: "Boolean", kind: self.typename() })
			}
		}
	}
}

impl ToNumber for Value {
	fn to_number(&self) -> crate::Result<Number> {
		unsafe {
			match self.tag() {
				Tag::Null => Null.to_number(),
				Tag::Boolean => self.as_boolean_unchecked().to_number(),
				Tag::Number => self.as_number_unchecked().to_number(),
				Tag::Text => self.as_text_unchecked().to_number(),
				_ => Err(Error::UndefinedConversion { into: "Number", kind: self.typename() })
			}
		}
	}
}

impl ToText for Value {
	fn to_text<'a>(&'a self) -> crate::Result<TextCow<'a>> {
		unsafe {
			match self.tag() {
				Tag::Null => Null.to_text(),
				// Tag::Boolean => self.as_boolean_unchecked().to_text(),
				Tag::Text => self.as_text_unchecked().as_text().to_text(),
				_ => todo!(),
			// 	Tag::Number => self.as_number_unchecked().to_text(),
				_ => Err(Error::UndefinedConversion { into: "Text", kind: self.typename() })
			}
		}
	}
}


/*
use crate::text::{ToText, Text, TextRef, TextCow};
use crate::number::{ToNumber, Number, NumberType};
use crate::boolean::{ToBoolean, Boolean};

	pub fn to_text(&self) -> Result<TextCow> {
		unsafe {
			//increase refcount for all these types, then just copy us.
			match self.tag() {
				Tag::Null => Null.to_text(),
			// | Tag::Boolean | Tag::Number => { /* do nothing */ },
				Tag::Variable => Variable::clone_in_place(self.ptr()),
				Tag::Text => Text::clone_in_place(self.ptr()),
				Tag::Ast => Ast::clone_in_place(self.ptr()),
				#[cfg(feature="custom-types")]
				Tag::Custom => todo!()
			}

			self.copy()

		}
		if let Some(textref) = self.as_text() {
			TextCow::Borrowed(textref)
		} else {
			TextCow::new( )
		}
		unsafe {

		}
		Cow::Borrowed(&self.as_text().unwrap())
	}
}
*/

impl From<Number> for Value {
	#[inline]
	fn from(number: Number) -> Self {
		Self::new_number(number)
	}
}

impl From<Text> for Value {
	#[inline]
	fn from(text: Text) -> Self {
		Self::new_text(text)
	}
}

impl From<Boolean> for Value {
	#[inline]
	fn from(bool: Boolean) -> Self {
		Self::new_boolean(bool)
	}
}

impl From<Ast> for Value {
	#[inline]
	fn from(ast: Ast) -> Self {
		Self::new_ast(ast)
	}
}

impl From<Variable> for Value {
	#[inline]
	fn from(var: Variable) -> Self {
		Self::new_variable(var)
	}
}
// todo: from null?

impl Clone for Value {
	fn clone(&self) -> Self {
		use std::mem::ManuallyDrop;

		unsafe {
			//increase refcount for all these types, then just copy us.
			match self.tag() {
				Tag::Null | Tag::Boolean | Tag::Number => { /* do nothing */ },
				Tag::Variable => Variable::clone_in_place(self.ptr()),
				Tag::Text => Text::clone_in_place(self.ptr()),
				Tag::Ast => Ast::clone_in_place(self.ptr()),
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
				Tag::Variable => Variable::drop_in_place(self.ptr()),
				Tag::Text => Text::drop_in_place(self.ptr()),
				Tag::Ast => Ast::drop_in_place(self.ptr()),
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

impl TryEq for Value {}
impl TryPartialEq for Value {
	type Error = Error;

	fn try_eq(&self, rhs: &Self) -> Result<bool> {
		#[cfg(feature="strict-compliance")] {
			if self.is_ast() || self.is_variable() {
				return Err(Error::Type { func: '?', operand: self.typename() });
			} else if rhs.is_ast() || rhs.is_variable() {
				return Err(Error::Type { func: '?', operand: rhs.typename() });
			}
		}

		if self.bytes() == rhs.bytes() {
			// literals have the same bytes.
			Ok(true)
		} else if cfg!(not(feature="cache-strings")) && self.is_text() && rhs.is_text() {
			// only texts can have equality without pointing to the same location.
			// note that if we're caching strings, then only identical pointers are equal.
			Ok(unsafe { &*self.as_text_unchecked() == &*rhs.as_text_unchecked() })
		} else {
			// todo: custom types comparison?

			// everything else is false.
			Ok(false)
		}
	}
}

impl TryOrd for Value {
	fn try_cmp(&self, rhs: &Self) -> Result<Ordering> {
		match self.classify() {
			ValueKind::Boolean(boolean) => Ok(boolean.cmp(&rhs.to_boolean()?)),
			ValueKind::Number(number) => Ok(number.cmp(&rhs.to_number()?)),
			ValueKind::Text(text) => Ok((&*text).cmp(&*rhs.to_text()?)),
			_ => Err(Error::Type { func: 'o', operand: self.typename() })
		}
	}
}

impl TryPartialOrd for Value {
	fn try_partial_cmp(&self, rhs: &Self) -> Result<Option<Ordering>> {
		self.try_cmp(rhs).map(Some)
	}

	fn try_lt(&self, rhs: &Self) -> Result<bool> {
		match self.try_cmp(rhs) {
			Err(Error::Type { operand, .. }) => Err(Error::Type { func: '<', operand }),
			other => other.map(|cmp| cmp == Ordering::Less)
		}
	}

	fn try_gt(&self, rhs: &Self) -> Result<bool> {
		match self.try_cmp(rhs) {
			Err(Error::Type { operand, .. }) => Err(Error::Type { func: '>', operand }),
			other => other.map(|cmp| cmp == Ordering::Greater)
		}
	}
}

impl TryAdd for &Value {
	type Error = Error;
	type Output = Value;

	fn try_add(self, rhs: Self) -> Result<Self::Output> {
		match self.classify() {
			ValueKind::Number(num) => Ok(num.try_add(rhs.to_number()?)?.into()),
			ValueKind::Text(text) => Ok((&*text + &*rhs.to_text()?).into()),
			_ => Err(Error::Type { func: '+', operand: self.typename() })
		}
	}
}

impl TrySub for &Value {
	type Error = Error;
	type Output = Value;

	fn try_sub(self, rhs: Self) -> Result<Self::Output> {
		match self.classify() {
			ValueKind::Number(num) => Ok(num.try_sub(rhs.to_number()?)?.into()),
			_ => Err(Error::Type { func: '-', operand: self.typename() })
		}
	}
}

impl TryMul for &Value {
	type Error = Error;
	type Output = Value;

	fn try_mul(self, rhs: Self) -> Result<Self::Output> {
		match self.classify() {
			ValueKind::Number(num) => Ok(num.try_mul(rhs.to_number()?)?.into()),
			ValueKind::Text(text) =>
				usize::try_from(rhs.to_number()?.get())
					.map(|amnt| &*text * amnt)
					.map(Value::from)
					.or(Err(Error::Domain("cannot multiply a string by a negative amount"))),
			_ => Err(Error::Type { func: '*', operand: self.typename() })
		}
	}
}

impl TryDiv for &Value {
	type Error = Error;
	type Output = Value;

	fn try_div(self, rhs: Self) -> Result<Self::Output> {
		match self.classify() {
			ValueKind::Number(num) => Ok(num.try_div(rhs.to_number()?)?.into()),
			_ => Err(Error::Type { func: '/', operand: self.typename() })
		}
	}
}

impl TryRem for &Value {
	type Error = Error;
	type Output = Value;

	fn try_rem(self, rhs: Self) -> Result<Self::Output> {
		match self.classify() {
			ValueKind::Number(num) => Ok(num.try_rem(rhs.to_number()?)?.into()),
			_ => Err(Error::Type { func: '%', operand: self.typename() })
		}
	}
}

impl TryPow for &Value {
	type Error = Error;
	type Output = Value;

	fn try_pow(self, rhs: Self) -> Result<Self::Output> {
		match self.classify() {
			ValueKind::Number(num) => Ok(num.try_pow(rhs.to_number()?)?.into()),
			_ => Err(Error::Type { func: '^', operand: self.typename() })
		}
	}
}