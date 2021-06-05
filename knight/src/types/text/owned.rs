#![allow(unused)]
use super::inner::TextInner;
use super::InvalidSourceByte;
use std::ptr::NonNull;

use crate::{Value, Boolean, Number};
use crate::ops::{Idempotent, ToNumber, ToBoolean, ToText, Infallible};
use crate::value::{Tag, ValueKind};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::borrow::{Cow, Borrow};
use std::fmt::{self, Debug, Display, Formatter};

#[repr(transparent)]
pub struct TextOwned(NonNull<TextInner>);

impl Clone for TextOwned {
	#[inline]
	fn clone(&self) -> Self {
		unsafe {
			TextInner::increment_refcount(self.0.as_ptr());
		}

		Self(self.0)
	}
}

impl Drop for TextOwned {
	#[inline]
	fn drop(&mut self) {
		unsafe {
			TextInner::decrement_refcount_maybe_dealloc(self.0.as_ptr())
		}
	}
}

impl Default for TextOwned {
	#[inline]
	fn default() -> Self {
		Self(TextInner::empty())
	}
}

impl Debug for TextOwned {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_tuple("TextOwned").field(&self.as_str()).finish()
	}
}

impl TextOwned {
	fn inner(&self) -> &TextInner {
		unsafe { &*self.0.as_ptr() }
	}
}

impl TextOwned {
	pub fn new(data: Cow<'static, str>) -> Result<Self, InvalidSourceByte> {
		super::validate_text(data.borrow())?;

		unsafe {
			Ok(Self::new_unchecked(data))
		}
	}

	// todo: remove this in favor of deref to `TextRef`.
	pub fn len(&self) -> usize {
		self.as_str().len()
	}

	#[inline]
	pub(super) unsafe fn from_inner(inner: NonNull<TextInner>) -> Self {
		Self(inner)
	}

	pub unsafe fn new_unchecked(data: Cow<'static, str>) -> Self {
		debug_assert_eq!(super::validate_text(data.borrow()), Ok(()));

		todo!()
	}

	pub fn as_str(&self) -> &str {
		self.inner().as_ref()
	}
}

// impl Text {
// 	pub fn new(data: Cow<'static, str>) -> Result<Self, InvalidSourceByte> {
// 		// todo
// 		unsafe {
// 			Ok(Self::new_unchecked(data))
// 		}
// 	}

// 	pub unsafe fn new_unchecked(data: Cow<'static, str>) -> Self {
// 		let inner = TextInner {
// 			rc: AtomicUsize::new(1),
// 			data,
// 			alloc: true
// 		};

// 		Self(NonNull::new_unchecked(Box::into_raw(Box::new(inner))))
// 	}

// 	fn inner(&self) -> &TextInner {
// 		unsafe { &*self.0.as_ptr() }
// 	}

// 	pub fn as_str(&self) -> &str {
// 		unsafe {
// 			(*self.0.as_ptr()).data.as_ref()
// 		}
// 	}

// 	pub fn len(&self) -> usize {
// 		self.as_str().len()
// 	}

// 	pub fn is_empty(&self) -> bool {
// 		self.len() == 0
// 	}

// 	pub(crate) unsafe fn drop_in_place(ptr: *mut ()) {
// 		let ptr = ptr as *mut TextInner;

// 		debug_assert_eq!((*ptr).rc.load(Ordering::Relaxed), 0);

// 		std::ptr::drop_in_place(ptr);
// 	}

// 	fn into_raw(self) -> *mut () {
// 		std::mem::ManuallyDrop::new(self).0.as_ptr() as _
// 	}

// 	pub fn as_ref(&self) -> TextRef<'_> {
// 		TextRef(self.inner())
// 	}
// }

// impl AsRef<str> for Text {
// 	fn as_ref(&self) -> &str {
// 		self.as_str()
// 	}
// }

// impl Eq for Text {}
// impl PartialEq for Text {
// 	#[inline]
// 	fn eq(&self, rhs: &Self) -> bool {
// 		self.as_str() == rhs.as_str()
// 	}
// }

// impl PartialEq<str> for Text {
// 	#[inline]
// 	fn eq(&self, rhs: &str) -> bool {
// 		self.as_str() == rhs
// 	}
// }

// impl PartialOrd for Text {
// 	#[inline]
// 	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
// 		Some(self.cmp(rhs))
// 	}
// }

// impl Ord for Text {
// 	#[inline]
// 	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
// 		self.as_str().cmp(rhs.as_str())
// 	}
// }

// impl From<Text> for Value<'_> {
// 	fn from(text: Text) -> Self {
// 		unsafe {
// 			Self::new_tagged(text.into_raw() as _, Tag::Text)
// 		}
// 	}
// }

// unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Text {
// 	type Ref = TextRef<'value>;

// 	fn is_value_a(value: &Value<'env>) -> bool {
// 		value.tag() == Tag::Text
// 	}

// 	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
// 		debug_assert!(Self::is_value_a(value));

// 		TextRef(&*value.ptr::<TextInner>().as_ptr())
// 	}
// }

// impl Idempotent<'_> for Text {}

// impl Display for Text {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
// 		Display::fmt(&self.as_str(), f)
// 	}
// }

// /// An error trait to indicate that [converting](<Number as TryFrom<Text>>::try_From) from a [`Text`] to a [`Number`]
// /// overflowed the maximum size for a number.
// #[derive(Debug)]
// pub struct NumberOverflow;

// impl Display for NumberOverflow {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
// 		write!(f, "string to number conversion overflowed the maximum number size!")
// 	}
// }

// impl std::error::Error for NumberOverflow {}

// impl<'a> ToText<'a> for Text {
// 	type Error = Infallible;
// 	type Output = TextRef<'a>;

// 	fn to_text(&'a self) -> Result<Self::Output, Self::Error> {
// 		Ok(self.as_ref())
// 	}
// }

// impl ToBoolean for Text {
// 	type Error = Infallible;

// 	fn to_boolean(&self) -> Result<Boolean, Self::Error> {
// 		Ok(!self.is_empty())
// 	}
// }

// impl ToNumber for Text {
// 	type Error = NumberOverflow;

// 	fn to_number(&self) -> Result<Number, Self::Error> {
// 		let mut iter = self.as_str().trim_start().bytes();
// 		let mut num = 0 as i64;
// 		let mut is_neg = false;

// 		match iter.next() {
// 			Some(b'-') => is_neg = true,
// 			Some(b'+') => { /* do nothing */ },
// 			Some(digit @ b'0'..=b'9') => num = (digit - b'0') as i64,
// 			_ => return Ok(Number::ZERO)
// 		}

// 		while let Some(digit) = iter.next() {
// 			if !digit.is_ascii_digit() {
// 				break;
// 			}

// 			let digit = (digit - b'0') as i64;

// 			if cfg!(feature="checked-overflow") {
// 				if let Some(new) = num.checked_mul(10).and_then(|n| n.checked_add(digit)) {
// 					num = new
// 				} else {
// 					return Err(NumberOverflow);
// 				}
// 			} else {
// 				num = num.wrapping_mul(10).wrapping_add(digit);
// 			}
// 		}

// 		if cfg!(feature="checked-overflow") {
// 			if is_neg { num.checked_neg() } else { Some(num) }.and_then(Number::new).ok_or(NumberOverflow)
// 		} else {
// 			Ok(Number::new_truncate(if is_neg { num.wrapping_neg() } else { num }))
// 		}
// 	}
// }
