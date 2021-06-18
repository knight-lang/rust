use std::sync::atomic::{AtomicUsize, Ordering};
use std::mem::{size_of};
use std::ptr::{addr_of_mut, NonNull};
use std::alloc::{self, Layout};

bitflags::bitflags! {
	struct TextFlags: usize {
		const NONE        = 0b0000;
		const EMBEDDED    = 0b0001;
		const SHOULD_FREE = 0b0010;
		const CACHED      = 0b0100;
	}
}

const TEXT_SIZE: usize = 64;
const PRELUDE_SIZE: usize = size_of::<AtomicUsize>() + size_of::<TextFlags>() + size_of::<usize>();
const EMBED_SIZE: usize = TEXT_SIZE - PRELUDE_SIZE;

#[repr(C, align(8))]
pub(super) struct TextInner {
	rc: AtomicUsize,
	flags: TextFlags,
	len: usize,
	data: TextInnerData
}


#[repr(packed)]
union TextInnerData {
	embed: [u8; EMBED_SIZE],
	ptr: *const u8 // todo: make sure that this doesn't have alignment conflicts
}

// TODO: verify that `EMPTY` isn't used incorrectly to introduce UB.
static mut EMPTY: TextInner = TextInner {
	rc: AtomicUsize::new(1),
	flags: TextFlags::EMBEDDED,
	len: 0,
	data: TextInnerData { embed: [0; EMBED_SIZE] }
};

impl TextInner {
	pub fn empty() -> NonNull<Self> {
		return unsafe { NonNull::from(&mut EMPTY) }
	}

	pub const unsafe fn new_static_from_str_unchecked(static_str: &'static str) -> Self {
		Self {
			rc: AtomicUsize::new(1),
			flags: TextFlags::NONE,
			len: static_str.len(),
			data: TextInnerData {
				ptr: static_str.as_ptr()
			}
		}
	}

	pub fn alloc(size: usize) -> NonNull<Self> {
		if unlikely!(size == 0) {
			return Self::empty();
		}

		// we use `alloc_zeroed` because the uninitialized memory might be written to.
		// later on we could solve that.
		unsafe {
			let inner = alloc::alloc_zeroed(Layout::new::<Self>()) as *mut Self;
			assert!(!inner.is_null());

			addr_of_mut!((*inner).rc).write(AtomicUsize::new(1));
			addr_of_mut!((*inner).len).write(size);

			if size <= EMBED_SIZE {
				addr_of_mut!((*inner).flags).write(TextFlags::EMBEDDED | TextFlags::SHOULD_FREE);
			} else {
				addr_of_mut!((*inner).flags).write(TextFlags::SHOULD_FREE);

				debug_assert_ne!(size, 0);
				let ptr = alloc::alloc_zeroed(Layout::from_size_align_unchecked(size, 1));
				debug_assert!(!ptr.is_null());

				addr_of_mut!((*inner).data.ptr).write(ptr);
			}

			NonNull::new_unchecked(inner)
		}
	}


	pub fn refcount(&self) -> usize {
		self.rc.load(Ordering::Relaxed)
	}

	// SAFETY: `inner` must be a valid poitner to `Self`.
	pub unsafe fn increment_refcount(inner: *const Self) {
		(*inner).rc.fetch_add(1, Ordering::Acquire);
	}

	// SAFETY: `inner` must be a valid poitner to `Self`, and must have been allocated via `Self::alloc`.
	pub unsafe fn decrement_refcount_maybe_dealloc(inner: *mut Self) {
		if (*inner).rc.fetch_sub(1, Ordering::Release) == 1 {
			Self::dealloc(inner)
		}
	}

	pub unsafe fn dealloc(inner: *mut Self) {
		if unlikely!(!(*inner).should_free()) {
			return;
		}

		debug_assert_eq!((*inner).refcount(), 0);

		Self::dealloc_unchecked(inner);
	}

	// same as `dealloc`, except it doesn't check for refcounts or allocatedness.
	// SAFETY:
	// - `inner` must be a valid `Self`
	// - `inner` must have the `SHOULD_FREE` flag set (ie wasn't created via `new_static_embedded`)
	// - `inner` should not be used after freeing.
	pub unsafe fn dealloc_unchecked(inner: *mut Self) {
		if unlikely!(!(*inner).is_embedded()) {
			let layout = Layout::from_size_align_unchecked((*inner).len(), 1);
			alloc::dealloc((*inner).data.ptr as *mut u8, layout);
		}

		alloc::dealloc(inner as *mut u8, Layout::new::<Self>())
	}

	#[inline]
	pub const fn len(&self) -> usize {
		self.len
	}

	#[inline]
	const fn is_embedded(&self) -> bool {
		self.flags.contains(TextFlags::EMBEDDED)
	}

	#[inline]
	pub(super) const fn should_free(&self) -> bool {
		self.flags.contains(TextFlags::SHOULD_FREE)
	}

	#[inline]
	pub fn as_ptr(&self) -> *const u8 {
		if self.is_embedded() {
			unsafe { self.data.embed.as_ptr() }
		} else {
			unsafe { self.data.ptr as *const u8 }
		}
	}

	#[inline]
	pub fn as_bytes(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(self.as_ptr(), self.len())
		}
	}

	#[inline]
	// safety: we can only call this on things that were allocated, ie not static.
	pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
		debug_assert!(self.should_free());

		if self.is_embedded() {
			self.data.embed.as_mut_ptr()
		} else {
			self.data.ptr as *mut u8
		}
	}
}

impl AsRef<str> for TextInner {
	#[inline]
	fn as_ref(&self) -> &str {
		unsafe {
			std::str::from_utf8_unchecked(self.as_bytes())
		}
	}
}

