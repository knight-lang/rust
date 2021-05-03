use std::fmt::{self, Display, Formatter};
use crate::{Text, Number};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Null; // no PartialOrd b/c knight says you cant compare null

impl Display for Null {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "null")
	}
}

impl From<Null> for bool {
	#[inline]
	fn from(_: Null) -> Self {
		false
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

		const NULL: TextStatic = unsafe { TextStatic::new_static_unchecked("null") };

		NULL.text()
	}
}

// impl<'env> crate::Runnable<'env> for Null {
// 	fn run(&self, env: &'env mut Environment<'_ ,'_ ,'_>) -> Result<Value> {
// 		Ok(Value::from(self))
// 	}
// }
