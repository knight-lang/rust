use crate::{Value, Boolean, Number};
use crate::value::{Tag, ValueKind, SHIFT};
use std::fmt::{self, Display, Formatter};
use crate::ops::{Idempotent, ToText, Infallible};
use crate::types::text::TextStatic;

/// The null type within Knight.
///
/// This notably doesn't implement [`Ord`]/[`PartialOrd`], as the Knight spec says that nulls cannot be compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Null;

impl Display for Null {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt("null", f)
	}
}

impl From<Null> for Value<'_> {
	/// Converts [`Null`] to its corresponding [`Value`] representation.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Value, Null};
	/// assert!(Value::from(Null).is_a::<Null>());
	/// ```
	#[inline]
	fn from(_null: Null) -> Self {
		Self::NULL
	}
}

impl Value<'_> {
	/// A shorthand for `Value::from(Null)`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Value, Null};
	/// assert!(Value::NULL.is_a::<Null>());
	/// ```
	// SAFETY: definition of `NULL`; doesn't overlap with any other constants. (Note `0 << SHIFt` and `2 << SHIFT` are
	// booleans.)
	pub const NULL: Self = unsafe { Value::new_tagged(1 << SHIFT, Tag::Constant) };
}

// SAFETY: 
// - `is_value_a` : Only returns true when we're `Value::NULL`, which is only created via `Value::from(Null)` (or the 
//   associated constant, they're identical).
// - `downcast_unchecked` : Wwhen passed a valid `Null` value, will always recover the original one.
unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Null {
	type Ref = Self;

	#[inline]
	fn is_value_a(value: &Value<'env>) -> bool {
		value.raw() == Value::NULL.raw()
	}

	#[inline]
	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value), "Null::downcast_unchecked ran with a bad value: {:#016x}", value.raw());
		let _ = value;

		Self
	}
}

impl Idempotent<'_> for Null {}

impl From<Null> for Number {
	/// Converting [`Null`] to a [`Number`] simply returns [zero](Number::ZERO).
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Number, Null};
	/// assert_eq!(Number::from(Null), Number::ZERO);
	/// ```
	#[inline]
	fn from(_null: Null) -> Self {
		Self::ZERO
	}
}

impl From<Null> for Boolean {
	/// Converting [`Null`] to a [`Boolean`] simply returns `false`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Null, Boolean};
	/// assert_eq!(Boolean::from(Null), false);
	/// ```
	#[inline]
	fn from(_null: Null) -> Self {
		false
	}
}

impl ToText<'_> for Null {
	type Error = Infallible;
	type Output = &'static TextStatic;

	/// Simply returns `"null"`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Null, Text, ops::ToText};
	/// assert_eq!(Null.to_text().unwrap().as_ref(), "null");
	/// ```
	fn to_text(&self) -> Result<Self::Output, Self::Error> {
		static NULL: TextStatic = unsafe { TextStatic::new_unchecked("null") };

		Ok(&NULL)
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use crate::types::*;
	use crate::function::NOOP;
	use crate::Environment;

	#[test]
	fn is_value_a() {
		assert!(Null::is_value_a(&Null.into()));
		assert!(!Null::is_value_a(&true.into()));
		assert!(!Null::is_value_a(&false.into()));
		assert!(!Null::is_value_a(&Text::new("A".into()).unwrap().into()));
		assert!(!Null::is_value_a(&Number::new(123).unwrap().into()));
		assert!(!Null::is_value_a(&Number::new(0).unwrap().into()));
		assert!(!Null::is_value_a(&Number::new(1).unwrap().into()));
		assert!(!Null::is_value_a(&Ast::new(&NOOP, vec![Null.into()].into_boxed_slice()).into()));

		let env = Environment::default();
		let foo = env.fetch_var("foo");
		assert!(!Null::is_value_a(&foo.into()));
		foo.set(Value::NULL);
		assert!(!Null::is_value_a(&foo.into()));
	}

	#[test]
	fn downcast_unchecked() {
		assert_eq!(unsafe { Null::downcast_unchecked(&Value::from(Null)) }, Null);
	}
}
