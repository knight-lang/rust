use super::{validate_text, TextInner, InvalidText, TextRef};
use std::ptr::NonNull;

use crate::{Value, Boolean, Number};
use crate::ops::{Idempotent, ToNumber, ToBoolean, ToText, Infallible};
use crate::value::{Tag, ValueKind};

use std::borrow::{Cow, Borrow};
use std::fmt::{self, Debug, Display, Formatter};

/// The text type within Knight.
///
/// Note that the text within Knight is immutable: As such, [`Text`] struct does not allow for mutation of its contents
/// (see [`TextBuilder`] for a way to build up [`Text`]s). This allows for numerous optimizations, including reference
/// counting and caching.
///
/// Due to the way that [`Value`] is designed, it's impossible to [`Value::downcast()`] into a [`Text`] directly---
/// instead, it'll go through a [`TextRef`], which dereferences to a [`Text`].
///
/// Normally, [`Text`]s will accept all UTF-8 valid strings (ie `str`)s. However, this is not required by the Knight
/// specs, and  is considered an extension. Thus, enabling the `disallow-unicode` feature will cause [`Text`]s to only
/// be constructable with valid Knight text bytes. See [`validate_text`] for more details.
#[repr(transparent)]
pub struct Text(NonNull<TextInner>);

impl Clone for Text {
	#[inline]
	fn clone(&self) -> Self {
		if cfg!(debug_assertions) && self.inner().should_free() {
			debug_assert_ne!(self.inner().refcount(), 0);
		}

		// SAFETY: we know that `self.0` is a valid `TextInner`, as we own it. This, it'll only ever be `free`d after all
		// of its refcounts are gone, which can only happen when there are no live references to `self`. Also, since we 
		// increment the refcount, we know that `from_inner` will be passed an "owned" reference.
		unsafe {
			TextInner::increment_refcount(self.0.as_ptr());

			Self::from_inner(self.0)
		}
	}
}

impl Drop for Text {
	#[inline]
	fn drop(&mut self) {
		if cfg!(debug_assertions) && self.inner().should_free() {
			debug_assert_ne!(self.inner().refcount(), 0);
		}

		// SAFETY: Since we allocated `self.0` via `TextInner::alloc`, and have not freed it yet, we know it's safe.
		// (we only free it here---when `drop`ping instances.)
		unsafe {
			TextInner::decrement_refcount_maybe_dealloc(self.0.as_ptr())
		}
	}
}

impl Default for Text {
	#[inline]
	fn default() -> Self {
		Self(TextInner::empty())
	}
}

impl Debug for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Text")
			.field(&self.as_str())
			.finish()
	}
}

impl Display for Text {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.as_str(), f)
	}
}

impl std::ops::Deref for Text {
	type Target = str;

	#[inline]
	fn deref(&self) -> &Self::Target {
		self.inner().as_ref()
	}
}

impl Text {
	// Gets a reference to the enclosed `TextInner`.
	pub(super) fn inner(&self) -> &TextInner {
		unsafe { &*self.0.as_ptr() }
	}

	/// Creates a new [`Text`] with the given string, returning an `Err` if it's [not valid](validate_text).
	///
	/// If a cached [`Text`] for `str` already exists, it will be used instead. Otherwise, `str` will be allocated and
	/// then used.
	///
	/// # Errors
	/// See [`validate_text`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::types::Text;
	/// let text = Text::new("foo".into()).unwrap();
	/// assert_eq!(text, *"foo");
	/// ```
	pub fn new(str: Cow<'_, str>) -> Result<Self, InvalidText> {
		validate_text(str.borrow())?;

		// SAFETY: we literally validated it.
		unsafe {
			Ok(Self::new_unchecked(str))
		}
	}

	/// Creates a new [`Text`], without [verifying that it is valid](validate_text).
	///
	/// # Safety
	/// It's up to the caller to ensure that `str` is a valid Knight string---ie[`validate_text(str)`](validate_text)
	/// will not return an `Err`.
	///
	/// # Examples
	/// ```rust
	/// ```
	pub unsafe fn new_unchecked(str: Cow<'_, str>) -> Self {
		debug_assert_eq!(validate_text(str.borrow()), Ok(()));

		let mut builder = Self::builder(str.as_ref().len());
		builder.write(str.borrow()).unwrap();
		builder.build()
	}


	#[inline]
	pub(super) unsafe fn from_inner(inner: NonNull<TextInner>) -> Self {
		Self(inner)
	}

	pub fn builder(capacity: usize) -> super::TextBuilder {
		super::TextBuilder::with_capacity(capacity)
	}

	pub fn as_str(&self) -> &str {
		&self
	}

	pub(crate) unsafe fn drop_in_place(ptr: *mut ()) {
		TextInner::dealloc(ptr as *mut TextInner);
	}

	pub(crate) fn should_free(&self) -> bool {
		self.inner().should_free()
	}

	fn into_raw(self) -> *mut () {
		std::mem::ManuallyDrop::new(self).0.as_ptr() as _
	}
}

// impl Text {
// 	pub fn new(data: Cow<'static, str>) -> Result<Self, InvalidText> {
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
	type Output = &'a Self;

	fn to_text(&'a self) -> Result<Self::Output, Self::Error> {
		Ok(self)
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

impl std::ops::Add<&str> for &Text {
	type Output = Text;

	fn add(self, rhs: &str) -> Self::Output {
		if rhs.is_empty() {
			return self.clone();
		}

		let mut builder = Text::builder(self.len() + rhs.len());

		builder.write(&self).unwrap();
		builder.write(rhs).unwrap();

		builder.build()
	}
}

impl std::ops::Mul<usize> for &Text {
	type Output = Text;

	fn mul(self, amnt: usize) -> Self::Output {
		let mut builder = Text::builder(self.len() * amnt);

		for _ in 0..amnt {
			builder.write(&self).unwrap();
		}

		builder.build()
	}
}
