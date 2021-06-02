use crate::{Value, Boolean, Number};
use crate::ops::{Runnable, ToNumber, ToBoolean, ToText, Infallible};
use crate::value::{Tag, ValueKind};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::borrow::{Cow, Borrow};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Deref, Add, Mul};
use std::ptr::NonNull;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Text(NonNull<TextInner>);

#[repr(C, align(8))]
struct TextInner {
	rc: AtomicUsize,
	data: Cow<'static, str>,
	alloc: bool
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct TextRef<'a>(&'a TextInner);

impl Clone for Text {
	fn clone(&self) -> Self {
		self.inner().rc.fetch_add(1, Ordering::Relaxed);
		Self(self.0)
	}
}

impl Drop for Text {
	fn drop(&mut self) {
		if !self.inner().alloc {
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
		static EMPTY: TextInner =
			TextInner { 
				rc: AtomicUsize::new(0),
				data: Cow::Borrowed(""),
				alloc: false
			};

		Self(NonNull::from(&EMPTY))
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

impl<'env> Runnable<'env> for Text {
	fn run(&self, _: &'env  crate::Environment) -> crate::Result<Value<'env>> {
		Ok(self.clone().into())
	}
}

impl Borrow<Text> for TextRef<'_> {
	fn borrow(&self) -> &Text {
		&self
	}
}

impl Deref for TextRef<'_> {
	type Target = Text;

	fn deref(&self) -> &Self::Target {
		// SAFETY:
		// `Text` is a transparent pointer to `TextInner` whereas `TextRef` is a transparent
		// reference to the same type. Since pointers and references can be transmuted safely, this is valid.
		unsafe {
			std::mem::transmute::<&TextRef<'_>, &Text>(self)
		}
	}
}

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


impl<T: AsRef<str>> Add<T> for TextRef<'_> {
	type Output = Text;

	fn add(self, rhs: T) -> Self::Output {
		let rhs = rhs.as_ref();

		if rhs.is_empty() {
			return (*self).clone();
		}

		let mut result = String::with_capacity(self.len() + rhs.len());
		result.push_str(self.as_str());
		result.push_str(rhs);

		Text::new(result.into()).unwrap()
	}
}

impl Mul<usize> for TextRef<'_> {
	type Output = Text;

	fn mul(self, amnt: usize) -> Self::Output {
		let mut result = String::with_capacity(self.len() * amnt);

		for _ in 0..amnt {
			result.push_str(self.as_str());
		}

		Text::new(result.into()).unwrap()
	}
}


