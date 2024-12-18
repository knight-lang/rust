use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::value2::{Value, ValueAlign};

/// Gc is the garbage collector for Knight [`Value`]s.
///
/// Layouts of allocated [`Value`]s are optimized to ensure that they all fit within
/// [`ALLOC_VALUE_SIZE`] bytes, which means they can easily be mass-allocated.
///
/// All allocated values are allocated via [`Gc::alloc_value_inner`].
#[must_use = "dropping `Gc` will leak all its memory"]
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
	flags: AtomicU8,
	// TODO: make this data maybeuninit
	data: [MaybeUninit<u8>; ALLOC_VALUE_SIZE - std::mem::size_of::<AtomicU8>()],
}

/// Indicates a value has been marked active during a mark-and-sweep.
const FLAG_GC_MARKED: u8 = 1 << 0;

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
	data: [MaybeUninit::uninit(); ALLOC_VALUE_SIZE - std::mem::size_of::<AtomicU8>()],
};

/// The options to give to [`Gc::new`]. More coming!
#[derive(Debug)]
#[non_exhaustive]
pub struct GcOptions {
	pub starting_cap: usize, // TODO
}

impl Default for GcOptions {
	fn default() -> Self {
		Self { starting_cap: 1000 }
	}
}

impl Gc {
	/// Constructs a new [`Gc`] with the given `opts`, and returns it.
	pub fn new(opts: GcOptions) -> Self {
		Self {
			value_inners: (0..opts.starting_cap)
				.map(|_| Box::into_raw(Box::new(EMPTY_INNER)))
				.collect(),
			roots: Vec::new(),
			idx: 0,
		}
	}

	/// Shuts down the [`Gc`] by cleaning up all memory associated with it.
	///
	/// # Safety
	/// Callers must ensure that no references to anything the [`Gc`] has created will be used after
	/// calling this function.
	pub unsafe fn shutdown(mut self) {
		for inner in self.value_inners {
			unsafe {
				ValueInner::deallocate(inner, false);
				drop(Box::from_raw(inner));
			}
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

	// pub unsafe fn alloc_value_inner3(&mut self, flags: u8) -> *mut ValueInner2<[u8; 10000]> {
	// 	unsafe { self.alloc_value_inner2::<[u8; 10000]>(flags) }
	// }

	// pub unsafe fn alloc_value_inner2<T>(&mut self, flags: u8) -> *mut ValueInner2<T> {
	// 	const {
	// 		let size = std::mem::size_of::<ValueInner2<T>>();
	// 		assert!(size <= ALLOC_VALUE_SIZE);
	// 		assert!(size <= ALLOC_VALUE_SIZE);
	// 	}
	// 	// };

	// 	// fn size<const N: usize>(_: [(); N]) {}
	// 	// size([(); std::mem::size_of::<ValueInner2<T>>()]);
	// 	// // const SIZE: usize = {
	// 	// // 	let size = ;
	// 	// // 	assert!(size <= ALLOC_VALUE_SIZE);
	// 	// // };

	// 	std::ptr::null_mut()
	// 	// debug_assert_eq!(flags & FLAG_GC_MARKED, 0, "cannot already be marked");
	// }

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

		let inner = self.next_open_inner();

		unsafe {
			(&raw mut (*inner).flags).write(AtomicU8::new(flags));
		}

		inner
	}

	/// Indicates that `root` is a "root node."
	///
	/// This adds `root` to a list of nodes that'll assume to always be "live," so them and all their
	/// children will be checked when marking-and-sweeping.
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

		// Sweep everything that's not needed
		for &inner in &self.value_inners {
			let old =
				unsafe { &*ValueInner::flags(inner) }.fetch_and(!FLAG_GC_MARKED, Ordering::SeqCst);

			debug_assert_eq!(old & FLAG_GC_STATIC, 0, "attempted to sweep a static flag?");

			// If it wasn't previously marked, then free it.
			if old & FLAG_GC_MARKED == 0 {
				unsafe {
					ValueInner::deallocate(inner, false);
				}
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
}

impl ValueInner {
	/// Gets the flags
	fn flags(this: *const Self) -> *const AtomicU8 {
		unsafe { &raw const (*this).flags }
	}

	pub(crate) unsafe fn as_knstring(this: *const Self) -> Option<crate::value2::KnString> {
		if unsafe { &*Self::flags(this) }.load(Ordering::SeqCst) & FLAG_IS_STRING != 0 {
			Some(unsafe { crate::value2::KnString::from_raw(this) })
		} else {
			None
		}
	}

	pub(crate) unsafe fn as_list(this: *const Self) -> Option<crate::value2::List> {
		if unsafe { &*Self::flags(this) }.load(Ordering::SeqCst) & FLAG_IS_LIST != 0 {
			Some(unsafe { crate::value2::List::from_raw(this) })
		} else {
			None
		}
	}

	pub(crate) unsafe fn mark(this: *const Self) {
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

	pub(crate) unsafe fn deallocate(this: *const Self, check: bool) {
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
