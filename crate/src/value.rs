use crate::{Function, Number, Text, Variable, Error, Result, Environment, Boolean, Ast};
use std::fmt::{self, Debug, Formatter};
use std::rc::Rc;
use std::convert::TryFrom;

#[derive(Clone)]
pub enum Value {
	Null,
	Boolean(Boolean),
	Number(Number),
	Text(Text),
	Variable(Variable),
	Ast(Ast)
}

impl Default for Value {
	#[inline]
	fn default() -> Self {
		Self::Null
	}
}

impl Eq for Value {}
impl PartialEq for Value {
	fn eq(&self, rhs: &Self) -> bool {
		match (self, rhs) {
			(Self::Null, Self::Null) => true,
			(Self::Boolean(lbool), Self::Boolean(rbool)) => lbool == rbool,
			(Self::Number(lnum), Self::Number(rnum)) => lnum == rnum,
			(Self::Text(ltext), Self::Text(rtext)) => ltext == rtext,
			(Self::Variable(lvar), Self::Variable(rvar)) => lvar == rvar,
			(Self::Ast(last), Self::Ast(rast)) => last == rast,
			_ => false
		}
	}
}

impl Debug for Value {
	// note we need the custom impl becuase `Null()` and `Identifier(...)` is required by the knight spec.
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Self::Null => write!(f, "Null()"),
			Self::Boolean(boolean) => write!(f, "Boolean({})", boolean),
			Self::Number(number) => write!(f, "Number({})", number),
			Self::Text(text) => write!(f, "Text({})", text),
			Self::Variable(variable) => write!(f, "Variable({})", variable.name()),
			Self::Ast(ast) => write!(f, "{:?}", ast)
		}
	}
}

impl From<Boolean> for Value {
	fn from(boolean: Boolean) -> Self {
		Self::Boolean(boolean)
	}
}

impl From<Number> for Value {
	fn from(number: Number) -> Self {
		Self::Number(number)
	}
}

impl From<Text> for Value {
	fn from(text: Text) -> Self {
		Self::Text(text)
	}
}

impl From<Variable> for Value {
	fn from(variable: Variable) -> Self {
		Self::Variable(variable)
	}
}


impl TryFrom<&Value> for Boolean {
	type Error = Error;

	fn try_from(value: &Value) -> Result<Self> {
		match value {
			Value::Null => Ok(false),
			Value::Boolean(boolean) => Ok(*boolean),
			Value::Number(number) => Ok(*number != 0),
			Value::Text(text) => Ok(!text.is_empty()),
			_ => Err(Error::UndefinedConversion { into: "Boolean", kind: value.typename() })
		}
	}
}

impl TryFrom<&Value> for Text {
	type Error = Error;

	fn try_from(value: &Value) -> Result<Self> {
		use once_cell::sync::OnceCell;

		static mut NULL: OnceCell<Text> = OnceCell::new();
		static mut TRUE: OnceCell<Text> = OnceCell::new();
		static mut FALSE: OnceCell<Text> = OnceCell::new();
		static mut ZERO: OnceCell<Text> = OnceCell::new();
		static mut ONE: OnceCell<Text> = OnceCell::new();

		match value {
			Value::Null => Ok(unsafe { &NULL }.get_or_init(|| unsafe { Self::new_unchecked("null") }).clone()),
			Value::Boolean(true) => Ok(unsafe { &TRUE }.get_or_init(|| unsafe { Self::new_unchecked("true") }).clone()),
			Value::Boolean(false) => Ok(unsafe { &FALSE }.get_or_init(|| unsafe { Self::new_unchecked("false") }).clone()),
			Value::Number(0) => Ok(unsafe { &ZERO }.get_or_init(|| unsafe { Self::new_unchecked("0") }).clone()),
			Value::Number(1) => Ok(unsafe { &ONE }.get_or_init(|| unsafe { Self::new_unchecked("1") }).clone()),
			Value::Number(number) => Ok(Self::try_from(number.to_string()).unwrap()), // all numbers should be valid strings
			Value::Text(text) => Ok(text.clone()),
			_ => Err(Error::UndefinedConversion { into: "Text", kind: value.typename() })
		}
	}
}

impl TryFrom<&Value> for Number {
	type Error = Error;

	fn try_from(value: &Value) -> Result<Self> {
		match value {
			Value::Null | Value::Boolean(false) => Ok(0),
			Value::Boolean(true) => Ok(1),
			Value::Number(number) => Ok(*number),
			Value::Text(text) => {
				let mut chars = text.trim().bytes();
				let mut sign = 1;
				let mut number: Number = 0;

				match chars.next() {
					Some(b'-') => sign = -1,
					Some(b'+') => { /* do nothing */ },
					Some(digit @ b'0'..=b'9') => number = (digit - b'0') as _,
					_ => return Ok(0)
				};

				while let Some(digit @ b'0'..=b'9') = chars.next() {
					cfg_if! {
						if #[cfg(feature="checked-overflow")] {
							number = number
								.checked_mul(10)
								.and_then(|num| num.checked_add((digit as u8 - b'0') as _))
								.ok_or(Error::TextConversionOverflow)?;
						} else {
							number = number.wrapping_mul(10).wrapping_add((digit as u8 - b'0') as _);
						}
					}
				}

				Ok(sign * number) // todo: check for this erroring. ?
			},
			_ => Err(Error::UndefinedConversion { into: "Number", kind: value.typename() })
		}
	}
}

impl Value {
	pub fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Self> {
		match self {
			Self::Null => Ok(Self::Null),
			Self::Boolean(boolean) => Ok(Self::Boolean(*boolean)),
			Self::Number(number) => Ok(Self::Number(*number)),
			Self::Text(text) => Ok(Self::Text(text.clone())),
			Self::Variable(variable) =>
				if let Some(var) = variable.fetch() {
					Ok(var)
				} else {
					handle_error!(Error::UnknownIdentifier { identifier: variable.name().into() })
				},
			Self::Ast(ast) => ast.run(env),
		}
	}

	#[must_use = "getting the type name by itself does nothing."]
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

	pub fn to_boolean(&self) -> Result<Boolean> {
		TryFrom::try_from(self)
	}

	pub fn to_number(&self) -> Result<Number> {
		TryFrom::try_from(self)
	}

	pub fn to_text(&self) -> Result<Text> {
		TryFrom::try_from(self)
	}
}
