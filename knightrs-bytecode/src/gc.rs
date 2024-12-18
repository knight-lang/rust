use std::sync::atomic::{AtomicU8, Ordering};

use crate::value2::{Value, ValueAlign};

pub struct Gc {
	value_inners: Vec<*mut ValueInner>,
	idx: usize,
	roots: Vec<Value>,
}

pub const ALLOC_VALUE_SIZE: usize = 32;

#[repr(C)]
pub struct ValueInner {
	_align: ValueAlign,
	// Note if `flags` is zero that means the field is unused. This won't happen when it's used
	// because the `IS_XXX` bits will be set, at a minimum.
	pub flags: AtomicU8,
	// TODO: make this data maybeuninit
	pub data: [u8; ALLOC_VALUE_SIZE - std::mem::size_of::<AtomicU8>()],
}

/// Indicates a value has been marked active during a mark-and-sweep.
pub const FLAG_GC_MARKED: u8 = 1 << 0;

/// Indicates a value is static, and shouldn't be a part of the GC cycle.
pub const FLAG_GC_STATIC: u8 = 1 << 1;

/// Indicates the [`ValueInner`] contains a [`KnString`].
pub const FLAG_IS_STRING: u8 = 1 << 2;

/// Indicates the [`ValueInner`] contains a [`List`].
pub const FLAG_IS_LIST: u8 = 1 << 3;

/// Indicates the [`ValueInner`] contains a custom type.
/// (Note: must check if `FLAG_IS_STRING` isn't set,as it uses FLAG_CUSTOM_0_DONTUSE)
pub const FLAG_IS_CUSTOM: u8 = 1 << 4;

/// An unused flag that types can use for their own purposes.
pub const FLAG_CUSTOM_0: u8 = 1 << 4;

/// An unused flag that types can use for their own purposes.
pub const FLAG_CUSTOM_1: u8 = 1 << 5;

/// An unused flag that types can use for their own purposes.
pub const FLAG_CUSTOM_2: u8 = 1 << 6;

/// An unused flag that types can use for their own purposes.
pub const FLAG_CUSTOM_3: u8 = 1 << 7;

const EMPTY_INNER: ValueInner = ValueInner {
	_align: ValueAlign,
	flags: AtomicU8::new(0),
	data: [0; ALLOC_VALUE_SIZE - std::mem::size_of::<AtomicU8>()],
};

impl Gc {
	pub fn new() -> Self {
		Self {
			value_inners: (0..1000).map(|_| Box::into_raw(Box::new(EMPTY_INNER))).collect(),
			roots: Vec::new(),
			idx: 0,
		}
	}

	fn next_open_inner_(&mut self) -> Option<*mut ValueInner> {
		while self.idx < self.value_inners.len() {
			let inner = self.value_inners[self.idx];
			self.idx += 1;
			if unsafe { &*ValueInner::flags(inner) }.load(Ordering::SeqCst) == 0 {
				return Some(inner);
			}
		}
		return None;
	}

	fn next_open_inner(&mut self) -> *mut ValueInner {
		if let Some(inner) = self.next_open_inner_() {
			return inner;
		}

		self.mark_and_sweep();
		self.idx = 0;

		// extend the length
		self
			.value_inners
			.extend((0..self.value_inners.len() + 1).map(|_| Box::into_raw(Box::new(EMPTY_INNER))));

		self.next_open_inner_().expect("we just extended")
	}

	/// Allocate another [`ValueInner`], possibly triggering a GC cycle if needed.
	///
	/// `flags` should contain the flags for the [`ValueInner`], and must:
	/// - Not contain [`FLAG_GC_MARKED`]
	/// - Contain [`FLAG_IS_CUSTOM`] or contain exactly one of [`FLAG_IS_STRING`] or [`FLAG_IS_LIST`].
	///
	/// # Safety
	/// Callers must ensure the above conditions are satisfied.
	pub unsafe fn alloc_value_inner(&mut self, flags: u8) -> *mut ValueInner {
		debug_assert_eq!(flags & FLAG_GC_MARKED, 0, "cannot already be marked");

		#[cfg(debug_assertions)]
		{
			let ty = flags & (FLAG_IS_STRING | FLAG_IS_LIST | FLAG_IS_CUSTOM);
			// If we have all of them set, or
			if ty == FLAG_IS_STRING || ty == (FLAG_IS_STRING | FLAG_IS_CUSTOM) {
			} else if ty == FLAG_IS_LIST || ty == (FLAG_IS_LIST | FLAG_IS_CUSTOM) {
			} else if ty == FLAG_IS_CUSTOM {
			} else {
				unreachable!("type passed in wasn't correct: {flags:08b}");
			}
		}

		unsafe {
			// TODO: use arena allocation
			let inner = Box::into_raw(Box::new(ValueInner {
				_align: ValueAlign,
				flags: AtomicU8::default(),
				data: [0; ALLOC_VALUE_SIZE - std::mem::size_of::<AtomicU8>()],
			}));

			if inner.is_null() {
				panic!("alloc failed");
			}

			(&raw mut (*inner).flags).write(AtomicU8::new(flags));
			self.value_inners.push(inner);
			inner
		}
	}

	/// Indicates that `root` is a "root node," to look through when sweeping.
	pub fn add_root(&mut self, root: Value) {
		self.roots.push(root);
	}

	fn mark_and_sweep(&mut self) {
		// Mark all elements accessible from the root
		for root in &mut self.roots {
			unsafe {
				root.mark();
			}
		}

		for &inner in &self.value_inners {
			let old =
				unsafe { &*ValueInner::flags(inner) }.fetch_and(!FLAG_GC_MARKED, Ordering::SeqCst);

			debug_assert_ne!(old & FLAG_GC_STATIC, 0, "attempted to sweep a static flag?");

			// If it wasn't previously marked, then free it.
			if old & FLAG_GC_MARKED == 0 {
				unsafe {
					ValueInner::deallocate(inner);
				}
			}
		}
	}

	pub unsafe fn shutdown(mut self) {
		for inner in self.value_inners {
			unsafe {
				ValueInner::deallocate_check(inner, false);
				drop(Box::from_raw(inner));
			}
		}
	}
}

/// A Trait implemented by types that are able to be garbage collected.
///
/// # Safety
/// Implementors must ensure that all of the methods are implemented correctly. More specifically,
/// they must ensure that:
///
/// - `mark`: _all_ [`GarbageCollected`] types reachable from `self` must be `mark`ed themselves.
//            If this is violated, then they might be freed before `self` is done with them.
pub unsafe trait GarbageCollected {
	/// Marks all the values reachable from `self` as "active."
	///
	/// Note that this is called after `self` has been marked itself, so only children reachable from
	/// `self` need to be marked.
	///
	/// # Safety
	/// This must only be called within a [`GarbageCollected::mark`] implementation. Calling it
	/// randomly will cause random things to be marked, which possibily will leak memory. (Which
	/// technically isn't undefined behaviour, but i've still marked it unsafe.)
	unsafe fn mark(&self);

	/// Frees all the memory related to `self`, **but not** memory related to [`GarbageCollected`]
	/// types `self` can access. (This is because they'll eventually have _thier_ `deallocate` called
	/// themselves.)
	///
	/// # Safety
	/// This shouldn't be called by anyone other than `GC.mark_and_sweep`, as there's no other way
	/// to ensure that nothing's used.
	unsafe fn deallocate(self);

	// unsafe fn sweep(&self, dealloc: bool)
}

// safety: has to make sure there's no cycle. shouldn't be for any builtin types.
/// Mark is a trait to represent
pub unsafe trait Mark {
	// safety: should not be called by anyone other than `gc`
	unsafe fn mark(&self);
}

pub trait Allocated {
	// safety: caller ensures `this` is the only reference
	unsafe fn deallocate(self);
}

pub unsafe trait Sweep {
	// safety: should not be called by anyone other than `gc`
	unsafe fn sweep(self, gc: &mut Gc);
}

// impl ValueInner {
impl ValueInner {
	// pub unsafe fn free(ptr: *mut Self) {
	// 	use std::alloc::{dealloc, Layout};
	// 	unsafe {
	// 		dealloc(ptr.cast::<u8>(), Layout::new::<Self>());
	// 	}
	// }

	fn flags(this: *const Self) -> *const AtomicU8 {
		unsafe { &raw const (*this).flags }
	}

	pub unsafe fn as_knstring(this: *const Self) -> Option<crate::value2::KnString> {
		if unsafe { &*Self::flags(this) }.load(Ordering::SeqCst) & FLAG_IS_STRING != 0 {
			Some(unsafe { crate::value2::KnString::from_value_inner(this) })
		} else {
			None
		}
	}

	pub unsafe fn as_list(this: *const Self) -> Option<crate::value2::List> {
		if unsafe { &*Self::flags(this) }.load(Ordering::SeqCst) & FLAG_IS_LIST != 0 {
			Some(unsafe { crate::value2::List::from_value_inner(this) })
		} else {
			None
		}
	}

	pub unsafe fn mark(this: *const Self) {
		let flags = unsafe { &*Self::flags(this) }.fetch_or(FLAG_GC_MARKED, Ordering::SeqCst);

		// Don't mark static things
		if flags & FLAG_GC_STATIC != 0 {
			return;
		}

		// If it was already marked, it's a loop, don't go again
		if flags & FLAG_GC_MARKED != 0 {
			dbg!("can we even loop?");
			return;
		}

		if let Some(mut list) = unsafe { Self::as_list(this) } {
			unsafe {
				list.mark();
			}
		}
	}

	pub unsafe fn sweep(this: *const Self) {
		let old = unsafe { &*Self::flags(this) }.fetch_and(!FLAG_GC_MARKED, Ordering::SeqCst);

		// If it's static then just return
		if old & FLAG_GC_STATIC != 0 {
			return;
		}

		// If it's not marked, ie `mark` didn't mark it, then deallocate it.
		if old & FLAG_GC_MARKED == 0 {
			unsafe {
				Self::deallocate(this);
			}
		}
	}

	pub unsafe fn deallocate(this: *const Self) {
		unsafe {
			Self::deallocate_check(this, true);
		}
	}

	unsafe fn deallocate_check(this: *const Self, check: bool) {
		debug_assert_eq!(unsafe { &*Self::flags(this) }.load(Ordering::SeqCst) & FLAG_GC_STATIC, 0);

		if let Some(string) = unsafe { Self::as_knstring(this) } {
			unsafe {
				string.deallocate();
			}
		} else if let Some(list) = unsafe { Self::as_list(this) } {
			unsafe {
				list.deallocate();
			}
		} else if check {
			unreachable!("non-list non-string encountered?");
		}

		// Mark it as `0` to indicate it's unused.
		unsafe { &*Self::flags(this) }.store(0, Ordering::SeqCst);
	}
}
