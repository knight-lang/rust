use crate::{Value, Boolean, Number};
use crate::ops::{Idempotent, ToNumber, ToBoolean, ToText, Infallible};
use crate::value::{Tag, ValueKind};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::borrow::Cow;
use std::fmt::{self, Debug, Display, Formatter};
use std::ptr::NonNull;

mod r#static;
mod r#ref;
pub use r#static::TextStatic;
pub use r#ref::TextRef;

/// The text type within Knight.
///
/// According to the specs, implementations are only required to accept a limited subset of ASCII. However, since Rust
/// only  TODO
#[repr(transparent)]
pub struct Text(NonNull<TextInner>);
// todo: rename `Text` to `TextOwned` or something and make `TextRef` into `Text`---ie have `TextRef` be equiv to `str`,
// and have all functions take it.


#[repr(C, align(8))]
struct TextInner {
	rc: AtomicUsize,
	data: Cow<'static, str>,
	alloc: bool
}

const_assert!(std::mem::align_of::<TextInner>() >= (1 << crate::value::SHIFT));

impl Clone for Text {
	#[inline]
	fn clone(&self) -> Self {
		self.inner().rc.fetch_add(1, Ordering::Relaxed);

		Self(self.0)
	}
}

impl Drop for Text {
	fn drop(&mut self) {
		if unlikely!(!self.inner().alloc) {
			return; // we just ignore unallocated things.
		}

		let rc = self.inner().rc.fetch_sub(1, Ordering::Relaxed);
		debug_assert_ne!(rc, 0);

		if rc == 1 {
			unsafe {
				Self::drop_in_place(self.0.as_ptr() as *mut ());
			}
		}
	}
}

impl Default for Text {
	fn default() -> Self {
		static EMPTY: TextStatic = unsafe { TextStatic::new_unchecked("") };

		EMPTY.as_text()
	}
}

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Text").field(&self.as_str()).finish()
	}
}


#[derive(Debug)]
pub struct InvalidSourceByte {

}

impl Text {
	pub fn new(data: Cow<'static, str>) -> Result<Self, InvalidSourceByte> {
		// todo
		unsafe {
			Ok(Self::new_unchecked(data))
		}
	}

	pub unsafe fn new_unchecked(data: Cow<'static, str>) -> Self {
		let inner = TextInner {
			rc: AtomicUsize::new(1),
			data,
			alloc: true
		};

		Self(NonNull::new_unchecked(Box::into_raw(Box::new(inner))))
	}

	fn inner(&self) -> &TextInner {
		unsafe { &*self.0.as_ptr() }
	}

	pub fn as_str(&self) -> &str {
		unsafe {
			(*self.0.as_ptr()).data.as_ref()
		}
	}

	pub fn len(&self) -> usize {
		self.as_str().len()
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub(crate) unsafe fn drop_in_place(ptr: *mut ()) {
		let ptr = ptr as *mut TextInner;

		debug_assert_eq!((*ptr).rc.load(Ordering::Relaxed), 0);

		std::ptr::drop_in_place(ptr);
	}

	fn into_raw(self) -> *mut () {
		std::mem::ManuallyDrop::new(self).0.as_ptr() as _
	}

	pub fn as_ref(&self) -> TextRef<'_> {
		TextRef(self.inner())
	}
}

impl AsRef<str> for Text {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl Eq for Text {}
impl PartialEq for Text {
	#[inline]
	fn eq(&self, rhs: &Self) -> bool {
		self.as_str() == rhs.as_str()
	}
}

impl PartialEq<str> for Text {
	#[inline]
	fn eq(&self, rhs: &str) -> bool {
		self.as_str() == rhs
	}
}

impl PartialOrd for Text {
	#[inline]
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(rhs))
	}
}

impl Ord for Text {
	#[inline]
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		self.as_str().cmp(rhs.as_str())
	}
}

impl From<Text> for Value<'_> {
	fn from(text: Text) -> Self {
		unsafe {
			Self::new_tagged(text.into_raw() as _, Tag::Text)
		}
	}
}

unsafe impl<'value, 'env: 'value> ValueKind<'value, 'env> for Text {
	type Ref = TextRef<'value>;

	fn is_value_a(value: &Value<'env>) -> bool {
		value.tag() == Tag::Text
	}

	unsafe fn downcast_unchecked(value: &'value Value<'env>) -> Self::Ref {
		debug_assert!(Self::is_value_a(value));

		TextRef(&*value.ptr::<TextInner>().as_ptr())
	}
}

impl Idempotent<'_> for Text {}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.as_str(), f)
	}
}

/// An error trait to indicate that [converting](<Number as TryFrom<Text>>::try_From) from a [`Text`] to a [`Number`]
/// overflowed the maximum size for a number.
#[derive(Debug)]
pub struct NumberOverflow;

impl Display for NumberOverflow {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "string to number conversion overflowed the maximum number size!")
	}
}

impl std::error::Error for NumberOverflow {}

impl<'a> ToText<'a> for Text {
	type Error = Infallible;
	type Output = TextRef<'a>;

	fn to_text(&'a self) -> Result<Self::Output, Self::Error> {
		Ok(self.as_ref())
	}
}

impl ToBoolean for Text {
	type Error = Infallible;

	fn to_boolean(&self) -> Result<Boolean, Self::Error> {
		Ok(!self.is_empty())
	}
}

impl ToNumber for Text {
	type Error = NumberOverflow;

	fn to_number(&self) -> Result<Number, Self::Error> {
		let mut iter = self.as_str().trim_start().bytes();
		let mut num = 0 as i64;
		let mut is_neg = false;

		match iter.next() {
			Some(b'-') => is_neg = true,
			Some(b'+') => { /* do nothing */ },
			Some(digit @ b'0'..=b'9') => num = (digit - b'0') as i64,
			_ => return Ok(Number::ZERO)
		}

		while let Some(digit) = iter.next() {
			if !digit.is_ascii_digit() {
				break;
			}

			let digit = (digit - b'0') as i64;

			if cfg!(feature="checked-overflow") {
				if let Some(new) = num.checked_mul(10).and_then(|n| n.checked_add(digit)) {
					num = new
				} else {
					return Err(NumberOverflow);
				}
			} else {
				num = num.wrapping_mul(10).wrapping_add(digit);
			}
		}

		if cfg!(feature="checked-overflow") {
			if is_neg { num.checked_neg() } else { Some(num) }.and_then(Number::new).ok_or(NumberOverflow)
		} else {
			Ok(Number::new_truncate(if is_neg { num.wrapping_neg() } else { num }))
		}
	}
}
