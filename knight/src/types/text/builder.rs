use super::{TextOwned, validate_text, InvalidText, inner::TextInner};
use std::ptr::{self, NonNull};

/// A Builder for [`TextOwned`]s.
///
/// As text within Knight is immutable, there's no way to incrementally build a [`TextOwned`]. This type is the solution
/// to that: You can incrementally write to the buffer via [`TextBuilder::write`] and finish it via
/// [`TextBuilder::build`]. However, you must write exactly the amount of bytes required, otherwise it'll `panic!`.
///
/// Note that all text written to the builder must still be [valid](super::validate_text).
///
/// # Examples
/// ```rust
/// use knight_lang::types::text::TextBuilder;
/// let mut builder = TextBuilder::with_capacity(6);
///
/// // `"foo"` and "`bar"` are valid for Knight strings.
/// assert!(builder.write("foo").is_ok());
/// assert!(builder.write("bar").is_ok());
///
/// // build it
/// let text = builder.build();
/// assert_eq!(text.as_str(), "foobar");
/// ```
#[must_use="Finalize builders via TextBuilder::build"]
pub struct TextBuilder {
	/// The `TextInner` that'll be written to and converted to a `TextOwned`.
	inner: NonNull<TextInner>,

	/// The amount of bytes written so far.
	len: usize,
}

impl Drop for TextBuilder {
	fn drop(&mut self) {
		// if the builder is dropped, we need to first write the remaining bytes, and then free it.
		self.write(&"&".repeat(self.bytes_remaining()));

		// SAFETY: 
		// - `inner` is a valid `TextInner`, as we allocated it with `with_capacity`.
		// - `inner` was not created via `new_static_embedded`, so it has the `SHOULD_FREE` flag enabled.
		// - `inner` will not be used after freeing, as this struct owns it.
		unsafe {
			TextInner::decrement_refcount_maybe_dealloc(self.inner.as_ptr())
		}
	}
}

impl TextBuilder {
	/// Creates a new [`TextBuilder`] with the given total capacity in bytes.
	///
	/// The builder will not resize itself to accommodate a larger capacity; instead, it will simply `panic!` if the
	/// capacity is overflown.
	///
	/// # Panics
	/// Panics if `capacity` is larger than `isize`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::types::text::TextBuilder;
	/// let builder = TextBuilder::with_capacity(6);
	///
	/// assert_eq!(builder.capacity(), 6);
	/// ```
	pub fn with_capacity(capacity: usize) -> Self {
		assert!(capacity <= isize::MAX as usize, "capacity is too large");

		Self {
			inner: TextInner::alloc(capacity),
			len: 0
		}
	}


	/// Fetches the length of the underlying buffer.
	///
	/// This is also the length of the resulting [`TextOwned`] after [`build()`](Self::build)ing `self`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::types::text::TextBuilder;
	/// let mut builder = TextBuilder::with_capacity(6);
	///
	/// assert_eq!(builder.capacity(), 6);
	///
	/// assert!(builder.write("foobar").is_ok());
	/// assert_eq!(builder.build().len(), 6);
	/// ```
	pub fn capacity(&self) -> usize {
		unsafe { self.inner.as_ref() }.len()
	}

	/// Fetches the amount of bytes that've been written to the [`TextBuilder`] so far.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::types::text::TextBuilder;
	/// let mut builder = TextBuilder::with_capacity(6);
	///
	/// // Starts off empty.
	/// assert_eq!(builder.len(), 0);
	///
	/// // After writing something to it, the `len` changes.
	/// assert!(builder.write("foo").is_ok());
	/// assert_eq!(builder.len(), 3);
	/// ```
	pub fn len(&self) -> usize {
		self.len
	}

	/// Fetches the amount of bytes that are left to write before you can [`build`](Self::build).
	///
	/// Note that attempting to [`write`](Self::write) after this value reaches `0` will cause a `panic!()`.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::types::text::TextBuilder;
	/// let mut builder = TextBuilder::with_capacity(6);
	///
	/// // Starts off equal to the `capacity`..
	/// assert_eq!(builder.bytes_remaining(), 6);
	///
	/// // After writing something to it, that many fewer bytes are required.
	/// assert!(builder.write("foob").is_ok());
	/// assert_eq!(builder.bytes_remaining(), 2);
	/// ```
	pub fn bytes_remaining(&self) -> usize {
		self.capacity() - self.len()
	}

	/// Concatenates `segment` to the end of the underlying buffer, increasing the [`len`](Self::len) appropriately.
	///
	/// # Errors
	/// If `segment` is not a [valid Knight text](validate_text), then a [`InvalidText`] is returned. Note that
	/// this is only returned if the `disallow-unicode` feature is enabled: Without it, all `segment`s are valid.
	///
	/// # Panics
	/// Panics if `segment`'s length is larger than [`bytes_remaining`].
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::types::text::TextBuilder;
	/// let mut builder = TextBuilder::with_capacity(6);
	///
	/// assert!(builder.write("foob").is_ok());
	/// assert!(builder.write("ar").is_ok());
	/// assert_eq!(builder.build().as_str(), "foobar");
	/// ```
	pub fn write(&mut self, segment: &str) -> Result<(), InvalidText> {
		assert!(segment.len() <= self.bytes_remaining(), "too many bytes written");
		validate_text(segment)?;

		unsafe {
			Ok(self.write_unchecked(segment.as_bytes()))
		}
	}

	// TODO: do we want a `try_write`? and if so, should it write as many bytes as possible, or only if all can fit.

	/// Concatenates `segment` to the end of the underlying buffer, without checking its length or validity.
	///
	/// # Safety
	/// It's up to the caller to ensure that `segment`'s length is at most [`bytes_remaining`] bytes long.
	///
	/// Additionally, the caller must ensure that the given `segment` is [valid Knight text](validate_text). This means
	/// that without the `disallow-unicode` feature enabled, `segment` must be valid UTF-8. With the feature, it must
	/// contain only valid Knight bytes.
	/// this is only returned if the `disallow-unicode` feature is enabled: Without it, all `segment`s are valid.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::types::text::TextBuilder;
	/// let mut builder = TextBuilder::with_capacity(6);
	///
	/// // SAFETY: `foo` and `bar` are both valid knight strings regardless of `disallow-unicode`.
	/// // additionally, `foo.len() + bar.len()` is equal to the starting capacity.
	/// unsafe {
	/// 	builder.write_unchecked(b"foo");
	/// 	builder.write_unchecked(b"bar");
	/// }
	///
	/// assert_eq!(builder.build().as_str(), "foobar");
	/// ```
	pub unsafe fn write_unchecked(&mut self, segment: &[u8]) {
		debug_assert!(segment.len() <= self.bytes_remaining());
		debug_assert!(validate_text(std::str::from_utf8(segment).unwrap()).is_ok());

		// Note that `self.len` is always a valid `isize`, as it's `<= capacity`, which is guaranteed to be no larger
		// than `isize::MAX`.
		let ptr = (*self.inner.as_ptr()).as_mut_ptr().offset(self.len() as isize);

		ptr::copy_nonoverlapping(segment.as_ptr(), ptr, segment.len());

		self.len += segment.len();
	}

	/// Creates a new [`TextOwned`] from the underlying buffer.
	///
	/// Note that this should only be called when the underlying buffer is completely full—that is, when
	/// [`bytes_remaining()`] is zero.
	///
	/// # Panics
	/// Panics if the [`bytes_remaining()`] is not zero.
	///
	/// [`bytes_remaining()`]: Self::bytes_remaining
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::types::text::TextBuilder;
	/// let mut builder = TextBuilder::with_capacity(6);
	///
	/// assert!(builder.write("foo").is_ok());
	/// assert!(builder.write("bar").is_ok());
	///
	/// assert_eq!(builder.build().as_str(), "foobar");
	/// ```
	#[must_use="`build` consumes the TextBuilder"]
	#[inline]
	pub fn build(self) -> TextOwned {
		assert_eq!(self.bytes_remaining(), 0, "underlying buffer is not full");

		unsafe { self.build_unchecked() }
	}

	/// Creates a new [`TextOwned`] from the underlying buffer, without verifying the buffer is fully written to.
	///
	/// # Safety
	/// It's up to the caller to ensure that [`bytes_remaining()`](Self::bytes_remaining) is zero.
	///
	/// # Examples
	/// ```rust
	/// # use knight_lang::types::text::TextBuilder;
	/// let mut builder = TextBuilder::with_capacity(6);
	///
	/// // SAFETY:
	/// // - `foo` and `bar` are both valid knight strings regardless of `disallow-unicode`.
	/// // - `foo.len() + bar.len()` is equal to the starting capacity.
	/// unsafe {
	/// 	builder.write_unchecked(b"foo");
	/// 	builder.write_unchecked(b"bar");
	/// 	assert_eq!(builder.build_unchecked().as_str(), "foobar");
	/// }
	/// ```
	#[must_use="`build` consumes the TextBuilder"]
	#[inline]
	pub unsafe fn build_unchecked(self) -> TextOwned {
		debug_assert_eq!(self.bytes_remaining(), 0, "not all bytes were written");

		// Manually drop so we don't free `inner` too.
		TextOwned::from_inner(std::mem::ManuallyDrop::new(self).inner)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn zero_capacity() {
		let mut builder = TextBuilder::with_capacity(0);

		assert_eq!(builder.len(), 0);
		assert_eq!(builder.capacity(), 0);
		assert_eq!(builder.bytes_remaining(), 0);
		assert_eq!(builder.write(""), Ok(()));

		assert_eq!(builder.len(), 0);
		assert_eq!(builder.capacity(), 0);
		assert_eq!(builder.bytes_remaining(), 0);
		assert_eq!(builder.build().as_str(), "");
	}

	#[test]
	fn normal_path() {
		let mut builder = TextBuilder::with_capacity(10);

		assert_eq!(builder.len(), 0);
		assert_eq!(builder.capacity(), 10);

		assert_eq!(builder.write("samp"), Ok(()));
		assert_eq!(builder.len(), 4);
		assert_eq!(builder.capacity(), 10);
		assert_eq!(builder.bytes_remaining(), 6);

		assert_eq!(builder.write("er"), Ok(()));
		assert_eq!(builder.len(), 6);
		assert_eq!(builder.capacity(), 10);
		assert_eq!(builder.bytes_remaining(), 4);

		// write nothing should still work.
		assert_eq!(builder.write(""), Ok(()));
		assert_eq!(builder.len(), 6);
		assert_eq!(builder.capacity(), 10);
		assert_eq!(builder.bytes_remaining(), 4);

		assert_eq!(builder.write("sand"), Ok(()));
		assert_eq!(builder.len(), 10);
		assert_eq!(builder.capacity(), 10);
		assert_eq!(builder.bytes_remaining(), 0);

		assert_eq!(builder.build().as_str(), "sampersand");
	}

	#[test]
	#[should_panic(expected="capacity is too large")]
	fn capacity_too_large() {
		let _ = TextBuilder::with_capacity(isize::MAX as usize + 1);
	}

	#[test]
	#[should_panic(expected="too many bytes written")]
	fn too_many_bytes_written_one_shot() {
		let mut builder = TextBuilder::with_capacity(3);
		let _ = builder.write("foobar");
	}

	#[test]
	#[should_panic(expected="too many bytes written")]
	fn too_many_bytes_written_overlapping() {
		let mut builder = TextBuilder::with_capacity(4);

		assert_eq!(builder.write("foo"), Ok(()));
		let _ = builder.write("bar");
	}

	#[test]
	#[should_panic(expected="too many bytes written")]
	fn too_many_bytes_written_exact() {
		let mut builder = TextBuilder::with_capacity(3);

		assert_eq!(builder.write("foo"), Ok(()));
		let _ = builder.write("bar");
	}

	#[test]
	#[should_panic(expected="underlying buffer is not full")]
	fn build_when_empty() {
		let builder = TextBuilder::with_capacity(3);

		let _ = builder.build();
	}

	#[test]
	#[should_panic(expected="underlying buffer is not full")]
	fn build_when_not_full() {
		let mut builder = TextBuilder::with_capacity(6);
		assert_eq!(builder.write("foo"), Ok(()));

		let _ = builder.build();
	}

	#[test]
	#[cfg(feature="disallow-unicode")]
	fn write_validates_text() {
		let mut builder = TextBuilder::with_capacity(15);

		assert_eq!(builder.write("hey"), Ok(()));
		assert_eq!(builder.len(), 3);
		assert_eq!(builder.capacity(), 15);
		assert_eq!(builder.bytes_remaining(), 12);

		assert!(builder.write("s☀️shine").is_err());
		assert_eq!(builder.len(), 3);
		assert_eq!(builder.capacity(), 15);
		assert_eq!(builder.bytes_remaining(), 12);

		assert!(builder.write("💖").is_err());
		assert_eq!(builder.len(), 3);
		assert_eq!(builder.capacity(), 15);
		assert_eq!(builder.bytes_remaining(), 12);

		assert_eq!(builder.write(" friend. :-D"), Ok(()));
		assert_eq!(builder.len(), 15);
		assert_eq!(builder.capacity(), 15);
		assert_eq!(builder.bytes_remaining(), 0);

		assert_eq!(builder.build().as_str(), "hey friend. :-D");
	}

	#[test]
	#[cfg(not(feature="disallow-unicode"))]
	fn write_allows_unicode() {
		let mut builder = TextBuilder::with_capacity(15);

		assert_eq!(builder.write("hi "), Ok(()));
		assert_eq!(builder.len(), 3);
		assert_eq!(builder.capacity(), 15);
		assert_eq!(builder.bytes_remaining(), 12);

		assert_eq!(builder.write("s☀️shine"), Ok(()));
		assert_eq!(builder.len(), 15);
		assert_eq!(builder.capacity(), 15);
		assert_eq!(builder.bytes_remaining(), 0);
		assert_eq!(builder.bytes_remaining(), 0);

		assert_eq!(builder.build().as_str(), "hi s☀️shine");
	}
}
