use crate::{Ast, Text, Variable, Custom, Null, Boolean, Number, Environment, Error};
use crate::ops::*;
use std::borrow::Borrow;
use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::marker::PhantomData;

pub(crate) const SHIFT: usize = 3;
const MASK: u8 = (1u8 << SHIFT) - 1;

/// The value type within Knight.
///
/// Since [`Value`]s can be [`Variable`]s, they must be associated with a given environment, hence the lifetime.
#[repr(transparent)]
pub struct Value<'env>(u64, PhantomData<&'env ()>);

/// A trait that represents the ability for something to be a [`Value`].
///
/// # Safety
/// Implementors of this trait must guarantee that calling `is_value_a` on a [`Value`] will only ever return `true` if
/// the given `value` was constructed via `<Self as Into<Value>>::into()`, and must not return `true` for any other
/// type.
pub unsafe trait ValueKind<'value, 'env: 'value> : Debug + Clone + Into<Value<'env>> + Runnable<'env> {
	/// The reference kind when [`downcast`](ValueKind::downcast_unchecked)ing a [`Value`].
	type Ref: Borrow<Self>;

	/// Checks to see if `value` is a `Self`.
	fn is_value_a(value: &Value<'env>) -> bool;

	/// Downcast the `value` to a [`Self::Ref`] without checking to see if `value` is a `Self`.
	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Tag {
	Constant = 0b000,
	Number   = 0b001,
	Variable = 0b010,
	Text     = 0b011,
	Ast      = 0b100,
	Custom   = 0b101,
}

impl Clone for Value<'_> {
	#[inline]
	fn clone(&self) -> Self {
		if !self.is_copy() {
			// SAFETY: All of `Variable`, `Text`, `Ast`, and `Custom`s have their first field as `AtomicUsize`s.
			unsafe {
				self.refcount().fetch_add(1, Ordering::Relaxed);
			}
		}

		// SAFETY: we just cloned all refcounted values, so we will have no double free errors.
		unsafe {
			self.copy()
		}
	}
}

impl Drop for Value<'_> {
	#[inline]
	fn drop(&mut self) {
		// we never want to inline this, as it's expensive to compute and not often called.
		#[cold]
		unsafe fn drop_inner(ptr: *mut (), tag: Tag) {
			match tag {
				Tag::Text => Text::drop_in_place(ptr),
				Tag::Ast => Ast::drop_in_place(ptr),
				Tag::Custom => Custom::drop_in_place(ptr),
				_ => unreachable!()
			}
		}

		if self.is_copy() {
			return;
		}

		let rc = unsafe { self.refcount() }.fetch_sub(1, Ordering::Relaxed);

		if cfg!(debug_assertions) {
			if let Some(text) = self.downcast::<Text>() {
				if text.should_free() {
					debug_assert_ne!(rc, 0);
				}
			} else {
				debug_assert_ne!(rc, 0);
			}
		}

		unsafe {
			if rc == 1 {
				drop_inner(self.ptr::<()>().as_ptr(), self.tag())
			}
		}
	}
}

impl Debug for Value<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		unsafe {
			match self.tag() {
				Tag::Constant if Null::is_value_a(self) => Debug::fmt(&Null, f),
				Tag::Constant => Debug::fmt(&Boolean::downcast_unchecked(self), f),
				Tag::Number => Debug::fmt(&Number::downcast_unchecked(self), f),
				Tag::Variable => Debug::fmt(&Variable::downcast_unchecked(self), f),
				Tag::Text => Debug::fmt(&*Text::downcast_unchecked(self), f),
				Tag::Ast => Debug::fmt(&*Ast::downcast_unchecked(self), f),
				Tag::Custom => Debug::fmt(&*Custom::downcast_unchecked(self), f),
			}
		}
	}
}

impl Default for Value<'_> {
	fn default() -> Self {
		Self::NULL
	}
}

impl<'env> Runnable<'env> for Value<'env> {
	fn run(&self, env: &'env  Environment) -> crate::Result<Value<'env>> {
		// in order of liklihood.
		if let Some(ast) = self.downcast::<Ast>() {
			ast.run(env)
		} else if let Some(variable) = self.downcast::<Variable>() {
			variable.run(env)
		} else if let Some(text) = self.downcast::<Text>() {
			text.run(env)
		} else {
			debug_assert!(matches!(self.tag(), Tag::Constant | Tag::Number));
			unsafe {
				Ok(self.copy())
			}
		}
	}
}

impl<'env> Value<'env> {
	// pub fn new<K: for<'a> ValueKind<'a>>(kind: K) -> Self {
	// 	kind.into_value()
	// }

	pub fn is_a<'a, T: ValueKind<'a, 'env>>(&'a self) -> bool {
		T::is_value_a(self)
	}

	pub unsafe fn downcast_unchecked<'a, T: ValueKind<'a, 'env>>(&'a self) -> T::Ref {
		debug_assert!(self.is_a::<T>());

		T::downcast_unchecked(self)
	}

	pub fn downcast<'a, T: ValueKind<'a, 'env>>(&'a self) -> Option<T::Ref> {
		if self.is_a::<T>() {
			unsafe {
				Some(self.downcast_unchecked::<T>())
			}
		} else {
			None
		}
	}

	// # SAFETY: `raw` must be a valid representation of a value.
	#[inline]
	pub(crate) const unsafe fn from_raw(raw: u64) -> Self {
		Self(raw, PhantomData)
	}

	// # SAFETY: `raw` must be a valid representation of a `tag`, and musn't have its lower bits set.
	#[inline]
	pub(crate) const unsafe fn new_tagged(raw: u64, tag: Tag) -> Self {
		debug_assert_eq_const!(raw & (MASK as u64), 0);

		Self::from_raw(raw | tag as u64)
	}

	#[inline(always)]
	pub(crate) const fn raw(&self) -> u64 {
		self.0
	}

	pub(crate) fn tag(&self) -> Tag {
		let raw_tag = (self.0 as u8) & MASK;

		debug_assert!(raw_tag <= Tag::Custom as u8);

		unsafe {
			std::mem::transmute::<u8, Tag>(raw_tag)
		}
	}

	pub(crate) const fn unmask(&self) -> u64 {
		self.0 & !(MASK as u64)
	}

	pub(crate) fn ptr<T>(&self) -> std::ptr::NonNull<T> {
		let ptr = self.unmask() as *mut T;
		debug_assert!(!ptr.is_null());

		unsafe {
			std::ptr::NonNull::new_unchecked(ptr)
		}
	}

	#[inline]
	fn is_copy(&self) -> bool {
		const_assert!((Tag::Constant as u64) <= 2);
		const_assert!((Tag::Number as u64) <= 2);
		const_assert!((Tag::Variable as u64) <= 2);
		const_assert!((Tag::Text as u64) > 2);
		const_assert!((Tag::Ast as u64) > 2);
		const_assert!((Tag::Custom as u64) > 2);

		(self.tag() as u64) <= 2
	}

	#[inline]
	fn is_literal(&self) -> bool {
		(self.tag() as u64) <= 1
	}

	pub fn is_idempotent(&self) -> bool {
		self.is_literal() || self.is_a::<Text>()
	}

	// SAFETY: must be a constant or a number.
	#[inline]
	unsafe fn copy(&self) -> Self {
		Self(self.0, PhantomData)
	}

	unsafe fn refcount(&self) -> &AtomicUsize {
		debug_assert!(!self.is_copy());

		&*self.ptr::<AtomicUsize>().as_ptr()
	}

	pub fn typename(&self) -> &'static str {
		match self.tag() {
			Tag::Constant if Null::is_value_a(self) => "Null",
			Tag::Constant => "Boolean",
			Tag::Number => "Number",
			Tag::Variable => "Variable",
			Tag::Text => "Text",
			Tag::Ast => "Ast",
			Tag::Custom => "Custom",
		}
	}
}

// n.b.: while we could use a `match` in these cases, we order them in liklihood.
impl<'env> ToNumber for Value<'env> {
	type Error = Error;

	fn to_number(&self) -> crate::Result<Number> {
		if let Some(number) = self.downcast::<Number>() {
			return Ok(number);
		}

		if let Some(textref) = self.downcast::<Text>() {
			return Ok(textref.to_number()?);
		}

		if self.is_literal() {
			return Ok(if self.raw() == Self::TRUE.raw() { Number::ONE } else { Number::ZERO });
		}

		unlikely!();
		Err(Error::UndefinedConversion { from: self.typename(), into: "Number" })
	}
}

impl<'env> ToText<'_> for Value<'env> {
	type Error = Error;
	type Output = Text;

	fn to_text(&self) -> crate::Result<Self::Output> {
		if let Some(text) = self.downcast::<Text>() {
			return Ok((*text).clone());
		}

		if let Some(number) = self.downcast::<Number>() {
			return Ok(number.to_text()?);
		}

		if let Some(boolean) = self.downcast::<Boolean>() {
			return Ok(boolean.to_text()?.as_ref().clone()); // TODO: not use a literal Text result.
		}

		if let Some(null) = self.downcast::<Null>() {
			return Ok(null.to_text()?.as_ref().clone()); // TODO: not use a literal Text result.
		}

		unlikely!();
		Err(Error::UndefinedConversion { from: self.typename(), into: "Text" })
	}
}

impl<'env> ToBoolean for Value<'env> {
	type Error = Error;

	fn to_boolean(&self) -> crate::Result<Boolean> {
		if self.is_literal() {
			let is_true = self.raw() > Self::NULL.raw();

			debug_assert_eq!(
				is_true,
				self.raw() != Self::NULL.raw()
					&& self.raw() != Self::FALSE.raw()
					&& self.raw() != Self::from(Number::ZERO).raw()
			);

			return Ok(is_true);
		}

		if self.is_a::<Text>() {
			unsafe {
				Ok(!self.downcast_unchecked::<Text>().is_empty())
			}
		} else {
			Err(Error::UndefinedConversion { from: self.typename(), into: "Boolean" })
		}
	}
}

impl<'env> TryAdd for Value<'env> {
	type Error = Error;
	type Output = Self;

	fn try_add(self, rhs: Self) -> crate::Result<Self> {
		if let Some(text) = self.downcast::<Text>() {
			let rhs = rhs.to_text()?;

			return Ok((&*text + rhs.as_str()).into());
		}

		if let Some(number) = self.downcast::<Number>() {
			return Ok(number.try_add(rhs.to_number()?)?.into())
		}

		unlikely!();
		Err(Error::InvalidArgument { func: '+', kind: self.typename() })
	}
}

impl<'env> TrySub for Value<'env> {
	type Error = Error;
	type Output = Self;

	fn try_sub(self, rhs: Self) -> crate::Result<Self> {
		if let Some(number) = self.downcast::<Number>() {
			return Ok(number.try_sub(rhs.to_number()?)?.into())
		}

		unlikely!();
		Err(Error::InvalidArgument { func: '-', kind: self.typename() })
	}
}

impl<'env> TryMul for Value<'env> {
	type Error = Error;
	type Output = Self;

	fn try_mul(self, rhs: Self) -> crate::Result<Self> {
		if let Some(text) = self.downcast::<Text>() {
			let rhs = rhs.to_number()?.get() as usize; // todo: check for usize.

			return Ok((&*text * rhs).into());
		}

		if let Some(number) = self.downcast::<Number>() {
			return Ok(number.try_mul(rhs.to_number()?)?.into())
		}

		unlikely!();
		Err(Error::InvalidArgument { func: '*', kind: self.typename() })
	}
}

impl<'env> TryDiv for Value<'env> {
	type Error = Error;
	type Output = Self;

	fn try_div(self, rhs: Self) -> crate::Result<Self> {
		if let Some(number) = self.downcast::<Number>() {
			return Ok(number.try_div(rhs.to_number()?)?.into())
		}

		unlikely!();
		Err(Error::InvalidArgument { func: '/', kind: self.typename() })
	}
}

impl<'env> TryRem for Value<'env> {
	type Error = Error;
	type Output = Self;

	fn try_rem(self, rhs: Self) -> crate::Result<Self> {
		if let Some(number) = self.downcast::<Number>() {
			return Ok(number.try_rem(rhs.to_number()?)?.into())
		}

		unlikely!();
		Err(Error::InvalidArgument { func: '%', kind: self.typename() })
	}
}

impl<'env> TryPow for Value<'env> {
	type Error = Error;
	type Output = Self;

	fn try_pow(self, rhs: Self) -> crate::Result<Self> {
		if let Some(number) = self.downcast::<Number>() {
			return Ok(number.try_pow(rhs.to_number()?)?.into())
		}

		unlikely!();
		Err(Error::InvalidArgument { func: '^', kind: self.typename() })
	}
}

impl<'env> TryNeg for Value<'env> {
	type Error = Error;
	type Output = Self;

	fn try_neg(self) -> crate::Result<Self> {
		if let Some(number) = self.downcast::<Number>() {
			return Ok(number.try_neg()?.into())
		}

		unlikely!();
		Err(Error::InvalidArgument { func: '~', kind: self.typename() })
	}
}

impl TryEq for Value<'_> {}
impl TryPartialEq for Value<'_> {
	type Error = Error;

	fn try_eq(&self, rhs: &Self) -> crate::Result<bool> {
		if cfg!(feature = "strict-compliance") {
			if !self.is_idempotent() {
				return Err(Error::InvalidArgument { func: '?', kind: self.typename() })
			}

			if !rhs.is_idempotent() {
				return Err(Error::InvalidArgument { func: '?', kind: rhs.typename() })
			}
		}

		if self.raw() == rhs.raw() {
			Ok(true)
		} else if let (Some(lhs), Some(rhs)) = (self.downcast::<Text>(), rhs.downcast::<Text>()) {
			Ok(*lhs == *rhs)
		} else {
			Ok(false)
		}
	}
}

impl TryPartialOrd for Value<'_> {
	fn try_partial_cmp(&self, rhs: &Self) -> crate::Result<Option<std::cmp::Ordering>> {
		self.try_cmp(rhs).map(Some)
	}
}

impl TryOrd for Value<'_> {
	fn try_cmp(&self, rhs: &Self) -> crate::Result<std::cmp::Ordering> {
		if let Some(text) = self.downcast::<Text>() {
			return Ok(text.cmp(&rhs.to_text()?));
		}

		if let Some(number) = self.downcast::<Number>() {
			return Ok(number.cmp(&rhs.to_number()?));
		}

		if let Some(boolean) = self.downcast::<Boolean>() {
			return Ok(boolean.cmp(&rhs.to_boolean()?));
		}

		// todo: how do we want to deal with `func` here?
		Err(Error::InvalidArgument { func: 'c', kind: self.typename() })
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! number {
		($number:expr) => (Number::new($number).unwrap());
	}

	macro_rules! text {
		($text:expr) => (Text::new($text.into()).unwrap());
	}

	macro_rules! ast {
		($which:ident $(, $values:expr)* $(,)?) => (Ast::new(&crate::function::$which, vec![$($values.into()),*].into_boxed_slice()));
	}

	#[test]
	fn is_literal() {
		assert!(Value::TRUE.is_literal());
		assert!(Value::FALSE.is_literal());
		assert!(Value::NULL.is_literal());
		assert!(Value::from(number!(0)).is_literal());
		assert!(Value::from(number!(1)).is_literal());
		assert!(Value::from(number!(-1)).is_literal());
		assert!(Value::from(number!(123)).is_literal());

		assert!(!Value::from(text!("")).is_literal());
		assert!(!Value::from(text!("A")).is_literal());

		assert!(!Value::from(ast![RANDOM]).is_literal());

		let env = Environment::default();
		assert!(!Value::from(env.fetch_var("foo")).is_literal());
	}

	#[test]
	#[ignore]
	fn run() {
		let env = Environment::default();

		assert_eq!(Value::NULL.run(&env).unwrap().downcast::<Null>(), Some(Null));

		assert_eq!(Value::TRUE.run(&env).unwrap().downcast::<Boolean>(), Some(true));
		assert_eq!(Value::FALSE.run(&env).unwrap().downcast::<Boolean>(), Some(false));

		assert_eq!(Value::from(number!(1)).run(&env).unwrap().downcast::<Number>(), Some(number!(1)));
		assert_eq!(Value::from(number!(0)).run(&env).unwrap().downcast::<Number>(), Some(number!(0)));
		assert_eq!(Value::from(number!(-1)).run(&env).unwrap().downcast::<Number>(), Some(number!(-1)));
		assert_eq!(Value::from(number!(123)).run(&env).unwrap().downcast::<Number>(), Some(number!(123)));
		assert_eq!(Value::from(number!(-12)).run(&env).unwrap().downcast::<Number>(), Some(number!(-12)));

		assert_eq!(*Value::from(text!("")).run(&env).unwrap().downcast::<Text>().unwrap(), text!(""));
		assert_eq!(*Value::from(text!("hello world")).run(&env).unwrap().downcast::<Text>().unwrap(), text!("hello world"));
		assert_eq!(*Value::from(text!("-123")).run(&env).unwrap().downcast::<Text>().unwrap(), text!("-123"));

		assert_eq!(Value::from(ast![NOOP, number!(4)]).run(&env).unwrap().downcast::<Number>(), Some(number!(4)));

		let env = Environment::default();
		let var = env.fetch_var("foo");
		var.set(number!(5).into());

		assert_eq!(Value::from(var).run(&env).unwrap().downcast::<Number>(), Some(number!(5)));
	}

	#[test]
	fn to_number() {
		assert_eq!(Value::NULL.to_number().unwrap(), Null.to_number().unwrap());

		assert_eq!(Value::TRUE.to_number().unwrap(), true.to_number().unwrap());
		assert_eq!(Value::FALSE.to_number().unwrap(), false.to_number().unwrap());

		assert_eq!(Value::from(number!(1)).to_number().unwrap(), number!(1).to_number().unwrap());
		assert_eq!(Value::from(number!(0)).to_number().unwrap(), number!(0).to_number().unwrap());
		assert_eq!(Value::from(number!(-1)).to_number().unwrap(), number!(-1).to_number().unwrap());
		assert_eq!(Value::from(number!(123)).to_number().unwrap(), number!(123).to_number().unwrap());
		assert_eq!(Value::from(number!(-12)).to_number().unwrap(), number!(-12).to_number().unwrap());

		assert_eq!(Value::from(text!("")).to_number().unwrap(), text!("").to_number().unwrap());
		assert_eq!(Value::from(text!("hello world")).to_number().unwrap(), text!("hello world").to_number().unwrap());
		assert_eq!(Value::from(text!("-123")).to_number().unwrap(), text!("-123").to_number().unwrap());

		assert_matches!(
			Value::from(ast![RANDOM]).to_number().unwrap_err(),
			Error::UndefinedConversion { from: "Ast", into: "Number" }
		);

		let env = Environment::default();
		assert_matches!(
			Value::from(env.fetch_var("foo")).to_number().unwrap_err(),
			Error::UndefinedConversion { from: "Variable", into: "Number" }
		);
	}

	#[test]
	fn to_text() {
		assert_eq!(Value::NULL.to_text().unwrap(), *Null.to_text().unwrap().as_ref());

		assert_eq!(Value::TRUE.to_text().unwrap(), *true.to_text().unwrap().as_ref());
		assert_eq!(Value::FALSE.to_text().unwrap(), *false.to_text().unwrap().as_ref());

		assert_eq!(Value::from(number!(1)).to_text().unwrap(), number!(1).to_text().unwrap());
		assert_eq!(Value::from(number!(0)).to_text().unwrap(), number!(0).to_text().unwrap());
		assert_eq!(Value::from(number!(-1)).to_text().unwrap(), number!(-1).to_text().unwrap());
		assert_eq!(Value::from(number!(123)).to_text().unwrap(), number!(123).to_text().unwrap());
		assert_eq!(Value::from(number!(-12)).to_text().unwrap(), number!(-12).to_text().unwrap());

		assert_eq!(Value::from(text!("")).to_text().unwrap(), *text!("").to_text().unwrap());
		assert_eq!(Value::from(text!("hello world")).to_text().unwrap(), *text!("hello world").to_text().unwrap());
		assert_eq!(Value::from(text!("-123")).to_text().unwrap(), *text!("-123").to_text().unwrap());

		assert_matches!(
			Value::from(ast![RANDOM]).to_text().unwrap_err(),
			Error::UndefinedConversion { from: "Ast", into: "Text" }
		);

		let env = Environment::default();
		assert_matches!(
			Value::from(env.fetch_var("foo")).to_text().unwrap_err(),
			Error::UndefinedConversion { from: "Variable", into: "Text" }
		);
	}

	#[test]
	fn to_boolean() {
		assert_eq!(Value::NULL.to_boolean().unwrap(), Null.to_boolean().unwrap());

		assert_eq!(Value::TRUE.to_boolean().unwrap(), true.to_boolean().unwrap());
		assert_eq!(Value::FALSE.to_boolean().unwrap(), false.to_boolean().unwrap());

		assert_eq!(Value::from(number!(1)).to_boolean().unwrap(), number!(1).to_boolean().unwrap());
		assert_eq!(Value::from(number!(0)).to_boolean().unwrap(), number!(0).to_boolean().unwrap());
		assert_eq!(Value::from(number!(-1)).to_boolean().unwrap(), number!(-1).to_boolean().unwrap());
		assert_eq!(Value::from(number!(123)).to_boolean().unwrap(), number!(123).to_boolean().unwrap());
		assert_eq!(Value::from(number!(-12)).to_boolean().unwrap(), number!(-12).to_boolean().unwrap());

		assert_eq!(Value::from(text!("")).to_boolean().unwrap(), text!("").to_boolean().unwrap());
		assert_eq!(Value::from(text!("hello world")).to_boolean().unwrap(), text!("hello world").to_boolean().unwrap());
		assert_eq!(Value::from(text!("-123")).to_boolean().unwrap(), text!("-123").to_boolean().unwrap());

		assert_matches!(
			Value::from(ast![RANDOM]).to_boolean().unwrap_err(),
			Error::UndefinedConversion { from: "Ast", into: "Boolean" }
		);

		let env = Environment::default();
		assert_matches!(
			Value::from(env.fetch_var("foo")).to_boolean().unwrap_err(),
			Error::UndefinedConversion { from: "Variable", into: "Boolean" }
		);
	}

	#[test]
	fn try_add() {}

	#[test]
	fn try_sub() {}
	
	#[test]
	fn try_mul() {}
	
	#[test]
	fn try_div() {}
	
	#[test]
	fn try_rem() {}
	
	#[test]
	fn try_pow() {}
	
	#[test]
	fn try_neg() {}
	
	#[test]
	fn try_cmp() {}

	#[test]
	fn try_eq() {}
}

