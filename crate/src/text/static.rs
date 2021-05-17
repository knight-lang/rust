use super::*;

use std::cell::UnsafeCell;

#[doc(hidden)]
#[repr(transparent)]
pub struct Combined<const N: usize>(UnsafeCell<CombinedInner<N>>);

#[repr(C, align(8))]
struct CombinedInner<const N: usize>(Inner, [u8; N]);

impl<const N: usize> Combined<N> {
	#[doc(hidden)]
	pub const fn new(data: [u8; N]) -> Self {
		Self(UnsafeCell::new(CombinedInner(Inner {
			len: N,
			#[cfg(all(not(feature="cache-strings"), not(feature="unsafe-single-threaded")))]
			rc: AtomicUsize::new(1),
			#[cfg(all(not(feature="cache-strings"), feature="unsafe-single-threaded"))]
			rc: 1,
			data: []
		},
		data)))
	}

	pub const unsafe fn text_for(&'static self) -> Text {
		Text(NonNull::new_unchecked(self.0.get() as *mut Inner))
	}
}

assert_eq_align!(Combined<0>, Inner);
assert_eq_size!(Combined<0>, Inner);

#[macro_export]
macro_rules! static_text {
	($text:literal) => {unsafe {
		use $crate::text::r#static::Combined;

		static mut INNER: Combined<{ $text.len() }> = Combined::new(*$text);
		INNER.text_for()
	}};
}
