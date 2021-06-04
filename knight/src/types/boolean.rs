use crate::{Value, Number};
use crate::ops::{Runnable, ToText, Infallible};
use crate::value::{SHIFT, ValueKind, Tag};
use crate::types::text::TextStatic;

/// The boolean type within Knight.
pub type Boolean = bool;

impl From<Boolean> for Value<'_> {
	/// Converts the `boolean` to its corresponding [`Value`] representation.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Value, Boolean};
	/// assert_eq!(Value::from(false).downcast::<Boolean>(), Some(false));
	/// assert_eq!(Value::from(true).downcast::<Boolean>(), Some(true));
	/// ```
	#[inline]
	fn from(boolean: Boolean) -> Self {
		// SAFETY:
		// - We shifted the boolean left, so we know the bottom `SHIFT` bits aren't set.
		// - the only two valid values for `boolean as u64` are 1 or 0, which when shifted left by `SHIFT + 1` yield
		//   Value::TRUE and Value::FALSE, respectively
		unsafe {
			// slight optimization lol
			Self::from_raw((boolean as u64) << (SHIFT + 1))
		}
	}
}

// SAFETY: 
// - `is_value_a` :  only returns true when we're literally `Value::FALSE` or `Value::TRUE`. Assuming all other
//   `ValueKind`s are well-defined, this will only return true for valid booleans.
// - `downcast_unchecked`, when passed a valid `Boolean` value, will always recover the original one.
unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Boolean {
	type Ref = Self;

	#[inline]
	fn is_value_a(value: &Value<'env>) -> bool {
		// Since `Value::TRUE` has exactly one bit set, the only way this can be zero is if the original value was zero
		// (ie it was `FALSE`), or if it were `Value::TRUE`'s bit.
		(value.raw() & !Value::TRUE.raw()) == 0
	}

	#[inline]
	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		value.raw() != Value::FALSE.raw()
	}
}

impl Value<'_> {
	/// A shorthand for `Value::from(false)`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Value, Boolean};
	/// assert_eq!(Value::FALSE.downcast::<Boolean>(), Some(false));
	/// ```
	pub const FALSE: Self = unsafe { Value::new_tagged(0, Tag::Constant) };

	/// A shorthand for `Value::from(true)`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Value, Boolean};
	/// assert_eq!(Value::TRUE.downcast::<Boolean>(), Some(true));
	/// ```
	pub const TRUE: Self = unsafe { Value::new_tagged(2 << SHIFT, Tag::Constant) };

}

impl<'env> Runnable<'env> for Boolean {
	/// Simply converts the [`Boolean`] to a [`Value`].
	/// 
	/// That is, [`run`](Self::run)ning a boolean is idempotent.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Environment, Value, Boolean, ops::Runnable};
	/// let env = Environment::default();
	///
	/// assert_eq!(true.run(&env).unwrap().downcast::<Boolean>(), Some(true));
	/// assert_eq!(false.run(&env).unwrap().downcast::<Boolean>(), Some(false));
	/// ```
	#[inline]
	fn run(&self, _env: &'env crate::Environment) -> crate::Result<Value<'env>> {
		Ok((*self).into())
	}
}

impl From<Boolean> for Number {
	/// Converts `false` to [zero](Number::ZERO) and `true` to [one](Number::ONE).
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::Number;
	/// assert_eq!(Number::from(true), Number::ONE);
	/// assert_eq!(Number::from(false), Number::ZERO);
	/// ```
	#[inline]
	fn from(boolean: Boolean) -> Self {
		// SAFETY: The only values for `bool as i64` are 1 and 0, both of which are valid `Number`s.
		unsafe {
			Self::new_unchecked(boolean as i64)
		}
	}
}

impl ToText<'_> for Boolean {
	type Error = Infallible;
	type Output = &'static TextStatic;

	/// Converts `false` to `"false"` and `true` to `"true"`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::{Text, ops::ToText};
	/// assert_eq!(true.to_text().unwrap().as_ref(), "true");
	/// assert_eq!(false.to_text().unwrap().as_ref(), "false");
	/// ```
	#[inline]
	fn to_text(&self) -> Result<Self::Output, Self::Error> {
		// SAFETY: both of these `TextStatic`s are created with valid knight strings.
		static TRUE: TextStatic = unsafe { TextStatic::new_unchecked("true") };
		static FALSE: TextStatic = unsafe { TextStatic::new_unchecked("false") };

		if *self {
			Ok(&TRUE)
		} else {
			Ok(&FALSE)
		}
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
		assert!(Boolean::is_value_a(&true.into()));
		assert!(Boolean::is_value_a(&false.into()));
		assert!(!Boolean::is_value_a(&Null.into()));
		assert!(!Boolean::is_value_a(&Text::new("A".into()).unwrap().into()));
		assert!(!Boolean::is_value_a(&Number::new(123).unwrap().into()));
		assert!(!Boolean::is_value_a(&Number::new(0).unwrap().into()));
		assert!(!Boolean::is_value_a(&Number::new(1).unwrap().into()));
		assert!(!Boolean::is_value_a(&Ast::new(&NOOP, vec![true.into()].into_boxed_slice()).into()));

		let env = Environment::default();
		let foo = env.fetch_var("foo");
		assert!(!Boolean::is_value_a(&foo.into()));
		foo.set(Value::TRUE);
		assert!(!Boolean::is_value_a(&foo.into()));
	}

	#[test]
	fn downcast_unchecked() {
		assert_eq!(unsafe { Boolean::downcast_unchecked(&Value::from(true)) }, true);
		assert_eq!(unsafe { Boolean::downcast_unchecked(&Value::from(false)) }, false);
	}
}
