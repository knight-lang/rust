use super::{Inner, Text};
use std::ptr::NonNull;

pub struct TextStatic(Inner);

impl TextStatic {
	pub fn text(&mut self) -> Text {
		unsafe {
			todo!()
			// Text(NonNull::new_unchecked(&self))
		}
	}
}

// #[repr(C, align(8))]
// struct Inner {
// 	len: usize,
// 	#[cfg(all(not(feature="cache-strings"), not(feature="unsafe-single-threaded")))]
// 	rc: AtomicUsize,
// 	#[cfg(all(not(feature="cache-strings"), feature="unsafe-single-threaded"))]
// 	rc: usize,
// 	data: [u8; 0]
// }

// // use super::{Inner, Text};
// // use std::marker::PhantomData;
// // use std::mem::ManuallyDrop;
// // use std::ptr::NonNull;

// // #[repr(transparent)]
// // #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// // pub struct TextRef<'a>(NonNull<Inner>, PhantomData<&'a ()>);

// // assert_eq_size!(TextRef, Text);
// // assert_eq_align!(TextRef, Text);

// // impl<'a> TextRef<'a> {
// // 	#[inline]
// // 	pub fn new(text: &'a Text) -> Self {
// // 		Self(text.0, PhantomData)
// // 	}

// // 	pub(crate) unsafe fn from_raw(raw: *const ()) -> Self {
// // 		Self(Text::from_raw(raw).0, PhantomData)
// // 	}

// // 	pub(crate) fn into_owned(self) -> Text {
// // 		(*self).clone()
// // 	}
// // }

// // impl std::ops::Deref for TextRef<'_> {
// // 	type Target = Text;

// // 	fn deref(&self) -> &Text {
// // 		unsafe {
// // 			&*(self as *const Self as *const Text)
// // 		}
// // 	}
// // }

// // impl std::borrow::Borrow<Text> for TextRef<'_> {
// // 	fn borrow(&self) -> &Text {
// // 		&self
// // 	}
// // }

// // impl AsRef<Text> for TextRef<'_> {
// // 	fn as_ref(&self) -> &Text {
// // 		&self
// // 	}
// // }
