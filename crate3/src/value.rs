use crate::{Null, Boolean, Text, TextRef, Number, Variable, Ast, Result, Environment, Error};
use std::num::NonZeroU64;
use std::marker::PhantomData;
use std::fmt::{self, Debug, Formatter};

pub struct Value(NonZeroU64);

pub enum ValueKind<'a> {
	Null,
	Boolean(Boolean),
	Number(Number),
	Text(TextRef<'a>),
	Variable(Variable),
	Ast(Ast)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tag {
	Constant = 1,
	Number = 2,
	Variable = 3,
	Text = 4,
	Ast = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Constant {
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
		match self.bytes() & TAG_MASK {
			1 => Tag::Constant,
			2 => Tag::Number,
			3 => Tag::Variable,
			4 => Tag::Text,
			5 => Tag::Ast,
			other => unsafe {
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

	pub fn classify(&self) -> ValueKind {
		unsafe {
			match self.tag() {
				Tag::Constant if self.is_null() => ValueKind::Null,
				Tag::Constant => ValueKind::Boolean(self.as_boolean_unchecked()),
				Tag::Number => ValueKind::Number(self.as_number_unchecked()),
				Tag::Text => ValueKind::Text(self.as_text_unchecked()),
				Tag::Variable => ValueKind::Variable(self.as_variable_unchecked()),
				Tag::Ast => ValueKind::Ast(self.as_ast_unchecked())
			}
		}
	}
}

impl From<Null> for Value {
	fn from(_: Null) -> Self {
		Self::NULL
	}
}

impl From<Boolean> for Value {
	fn from(bool: Boolean) -> Self {
		if bool {
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
			Self::new_tagged((number.inner() as u64) << TAG_BITS, Tag::Number)
		}
	}
}

impl From<Variable> for Value {
	fn from(variable: Variable) -> Self {
		unsafe {
			Self::new_tagged(variable.into_raw() as usize as u64, Tag::Variable)
		}
	}
}

impl From<Ast> for Value {
	fn from(ast: Ast) -> Self {
		unsafe {
			Self::new_tagged(ast.into_raw() as usize as u64, Tag::Ast)
		}
	}
}

impl ValueKind<'_> {
	pub const fn typename(&self) -> &'static str {
		match self {
			Self::Null => "Null",
			Self::Boolean(_) => "Boolean",
			Self::Number(_) => "Number",
			Self::Text(_) => "Text",
			Self::Variable(_) => "Variable",
			Self::Ast(_) => "Ast",
		}
	}
}

impl Value {
	pub const fn typename(&self) -> &'static str {
		match self.tag() {
			Tag::Constant if self.is_null() => "Null",
			Tag::Constant => "Boolean",
			Tag::Number => "Number",
			Tag::Text => "Text",
			Tag::Variable => "Variable",
			Tag::Ast => "Ast",
		}
	}

	pub const fn is_null(&self) -> bool {
		let x = Self::NULL;
		let r = self.0.get() == x.bytes();
		std::mem::forget(x);
		r
	}

	pub fn as_boolean(&self) -> Option<Boolean> {
		if self.bytes() == Self::TRUE.bytes() {
			Some(true)
		} else if self.bytes() == Self::FALSE.bytes() {
			Some(false)
		} else {
			None
		}
	}

	pub unsafe fn as_boolean_unchecked(&self) -> Boolean {
		debug_assert_const!(self.bytes() == Self::TRUE.bytes() || self.bytes() == Self::FALSE.bytes());

		self.bytes() == Self::TRUE.bytes()
	}

	pub fn as_number(&self) -> Option<Number> {
		if self.is_tag(Tag::Number) {
			Some(unsafe { self.as_number_unchecked() })
		} else {
			None
		}
	}

	pub unsafe fn as_number_unchecked(&self) -> Number {
		debug_assert_eq_const!(self.tag() as u64, Tag::Number as u64);

		Number::new_unchecked((self.bytes() as i64) >> TAG_BITS)
	}

	pub fn as_text(&self) -> Option<TextRef> {
		if self.is_tag(Tag::Text) {
			Some(unsafe { self.as_text_unchecked() })
		} else {
			None
		}
	}

	pub unsafe fn as_text_unchecked(&self) -> TextRef {
		debug_assert_eq!(self.tag(), Tag::Text);

		TextRef::from_raw(self.unmask() as _)
	}

	pub unsafe fn into_text_unchecked(&self) -> Text {
		debug_assert_eq!(self.tag(), Tag::Text);

		Text::from_raw(self.unmask() as _)
	}

	pub fn as_variable(&self) -> Option<Variable> {
		if self.is_tag(Tag::Variable) {
			Some(unsafe { self.as_variable_unchecked() })
		} else {
			None
		}
	}

	pub unsafe fn as_variable_unchecked(&self) -> Variable {
		debug_assert_eq!(self.tag(), Tag::Variable);

		Variable::from_raw(self.unmask() as _)
	}

	pub fn as_ast(&self) -> Option<Ast> {
		if self.is_tag(Tag::Ast) {
			Some(unsafe { self.as_ast_unchecked() })
		} else {
			None
		}
	}

	pub unsafe fn as_ast_unchecked(&self) -> Ast {
		debug_assert_eq!(self.tag(), Tag::Ast);

		Ast::from_raw(self.unmask() as _)
	}


	pub fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Self> {
		if let Some(ast) = self.as_ast() {
			ast.run(env)
		} else if let Some(var) = self.as_variable() {
			var.run(env)
		} else {
			Ok(self.clone())
		}
	}

	pub fn to_text(&self) -> Result<Text> {
		match self.classify() {
			ValueKind::Null => Ok(Null.into()),
			ValueKind::Boolean(boolean) => Ok(boolean.into()),
			ValueKind::Number(number) => Ok(number.into()),
			ValueKind::Text(ref text) => Ok((**text).clone()),
			ref kind => Err(Error::UndefinedConversion { to: "Text", from: kind.typename() })
		}
	}

	pub fn to_number(&self) -> Result<Number> {
		match self.classify() {
			ValueKind::Null => Ok(Null.into()),
			ValueKind::Boolean(boolean) => Ok(boolean.into()),
			ValueKind::Number(number) => Ok(number),
			ValueKind::Text(ref text) => text.parse().map_err(|err| Error::Custom(Box::new(err))),
			ref kind => Err(Error::UndefinedConversion { to: "Number", from: kind.typename() })
		}
	}

	pub fn to_boolean(&self) -> Result<Boolean> {
		match self.classify() {
			ValueKind::Null => Ok(Null.into()),
			ValueKind::Boolean(boolean) => Ok(boolean),
			ValueKind::Number(number) => Ok(number.into()),
			ValueKind::Text(ref text) => Ok(!text.is_empty()),
			ref kind => Err(Error::UndefinedConversion { to: "Boolean", from: kind.typename() })
		}
	}
}


impl Clone for Value {
	fn clone(&self) -> Self {
		match self.classify() {
			ValueKind::Null => Null.into(),
			ValueKind::Boolean(boolean) => boolean.into(),
			ValueKind::Number(number) => number.into(),
			ValueKind::Text(ref text) => (**text).clone().into(),
			ValueKind::Variable(ref var) => var.clone().into(),
			ValueKind::Ast(ref ast) => ast.clone().into(),
		}
	}
}

impl Drop for Value {
	fn drop(&mut self) {
		match self.tag() {
			Tag::Constant | Tag::Number => { /* do nothing */ },
			Tag::Text => unsafe { drop(self.into_text_unchecked()) },
			Tag::Variable => unsafe { drop(self.as_variable_unchecked()) },
			Tag::Ast => unsafe { drop(self.as_ast_unchecked()) }
		}
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut t = f.debug_tuple("Value");

		match self.classify() {
			ValueKind::Null => t.field(&Null).finish(),
			ValueKind::Boolean(boolean) => t.field(&boolean).finish(),
			ValueKind::Number(number) => t.field(&number).finish(),
			ValueKind::Text(ref text) => t.field(&text).finish(),
			ValueKind::Variable(variable) => t.field(&variable).finish(),
			ValueKind::Ast(ast) => t.field(&ast).finish(),
		}
	}
}

impl Eq for Value {}
impl PartialEq for Value {
	fn eq(&self, rhs: &Self) -> bool {
		self.0 == rhs.0 || self.as_text().map_or(false, |l| rhs.as_text().map_or(false, |r| l == r))
	}
}
