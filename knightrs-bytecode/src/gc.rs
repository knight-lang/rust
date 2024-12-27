use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{self, Debug, Formatter};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::value::{Value, ValueAlign};

/// Gc is the garbage collector for Knight [`Value`]s.
///
/// Layouts of allocated [`Value`]s are optimized to ensure that they all fit within
/// [`ALLOC_VALUE_SIZE`] bytes, which means they can easily be mass-allocated.
///
/// All allocated values are allocated via [`Gc::alloc_value_inner`].
#[must_use = "dropping `Gc` will leak all its memory"]
pub struct Gc(RefCell<Inner>);

struct Inner {
	value_inners: Vec<*mut ValueInner>,
	idx: usize,
	roots: HashSet<*const ValueInner>,
	paused: bool,
	mark_fns: HashMap<usize, Box<dyn Fn()>>,
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

impl Default for Gc {
	fn default() -> Self {
		Self::new(Default::default())
	}
}

impl Gc {
	/// Constructs a new [`Gc`] with the given `opts`, and returns it.
	pub fn new(opts: GcOptions) -> Self {
		Self(
			Inner {
				value_inners: (0..opts.starting_cap)
					.map(|_| Box::into_raw(Box::new(EMPTY_INNER)))
					.collect(),
				roots: HashSet::new(),
				idx: 0,
				paused: false,
				mark_fns: HashMap::new(),
			}
			.into(),
		)
	}

	// SAFETY: caller has to ensure that nothing allocated by `gc` escapes.
	pub unsafe fn run<T>(self, func: impl FnOnce(&Self) -> T) -> T {
		let result = func(&self);
		unsafe {
			self.shutdown();
		}
		result
	}

	pub fn del_mark_fn(&self, index: usize) {
		let _ = self.0.borrow_mut().mark_fns.remove(&index).expect("mark fn already removed");
	}

	pub fn add_mark_fn(&self, func: impl Fn() + 'static) -> usize {
		let mut inner = self.0.borrow_mut();
		let len = inner.mark_fns.len();
		inner.mark_fns.insert(len, Box::new(func));
		len
	}

	/// Shuts down the [`Gc`] by cleaning up all memory associated with it.
	///
	/// # Safety
	/// Callers must ensure that no references to anything the [`Gc`] has created will be used after
	/// calling this function.
	unsafe fn shutdown(self) {
		// TODO: this borrow isnt sound
		for &inner in &self.0.borrow().value_inners {
			unsafe {
				ValueInner::deallocate(inner, false);
				drop(Box::from_raw(inner));
			}
		}
	}

	fn next_open_inner_(&self) -> Option<*mut ValueInner> {
		let mut inner = self.0.borrow_mut();
		while inner.idx < inner.value_inners.len() {
			let value_inner = inner.value_inners[inner.idx];
			inner.idx += 1;
			if unsafe { &*ValueInner::flags(value_inner) }.load(Ordering::SeqCst) == 0 {
				return Some(value_inner);
			}
		}
		return None;
	}

	fn next_open_inner(&self) -> *mut ValueInner {
		if let Some(inner) = self.next_open_inner_() {
			return inner;
		}

		// TODO
		if !self.0.borrow().paused && false {
			unsafe {
				self.mark_and_sweep();
			}
			self.0.borrow_mut().idx = 0;
		}

		// extend the length
		let len = self.0.borrow().value_inners.len();
		self
			.0
			.borrow_mut()
			.value_inners
			.extend((0..=len).map(|_| Box::into_raw(Box::new(EMPTY_INNER))));

		self.next_open_inner_().expect("we just extended")
	}

	pub fn pause(&self) {
		let mut inner = self.0.borrow_mut();
		assert!(!inner.paused);
		inner.paused = true;
	}

	pub fn unpause(&self) {
		let mut inner = self.0.borrow_mut();
		assert!(inner.paused);
		inner.paused = false;
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
	pub unsafe fn alloc_value_inner(&self, flags: u8) -> *mut ValueInner {
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

		#[cfg(debug_assertions)] // always sweep after every allocation when testing
		if !self.0.borrow().paused && false {
			unsafe {
				self.mark_and_sweep();
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
	pub fn add_root(&self, root: Value) {
		if root.__is_alloc() {
			self.0.borrow_mut().roots.insert(unsafe { root.__as_alloc() });
		}
	}

	// pub only for testing
	pub unsafe fn mark_and_sweep(&self) {
		for mark_fn in self.0.borrow().mark_fns.values() {
			mark_fn()
		}

		// Mark all elements accessible from the root
		for &root in &self.0.borrow().roots {
			unsafe {
				ValueInner::mark(root);
			}
		}

		// Sweep everything that's not needed
		for &inner in &self.0.borrow().value_inners {
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

	pub(crate) unsafe fn as_knstring<'gc>(this: *const Self) -> Option<crate::value::KnString<'gc>> {
		if unsafe { &*Self::flags(this) }.load(Ordering::SeqCst) & FLAG_IS_STRING != 0 {
			Some(unsafe { crate::value::KnString::from_raw(this) })
		} else {
			None
		}
	}

	pub(crate) unsafe fn as_list<'gc>(this: *const Self) -> Option<crate::value::List<'gc>> {
		if unsafe { &*Self::flags(this) }.load(Ordering::SeqCst) & FLAG_IS_LIST != 0 {
			Some(unsafe { crate::value::List::from_raw(this) })
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

		if let Some(list) = unsafe { Self::as_list(this) } {
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

pub unsafe trait AsValueInner {
	fn as_value_inner(&self) -> *const ValueInner;
	unsafe fn from_value_inner(inner: *const ValueInner) -> Self;
}

pub struct GcRoot<'gc, T: AsValueInner>(T, Option<&'gc Gc>);

impl<T: AsValueInner + Debug> Debug for GcRoot<'_, T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Debug::fmt(&self.0, f)
	}
}

impl<'gc, T: AsValueInner> GcRoot<'gc, T> {
	// safety: that from_value_inner seems like it could be unsafe potentially lol
	pub fn new(t: &T, gc: &'gc Gc) -> Self {
		let inner = t.as_value_inner();
		gc.0.borrow_mut().roots.insert(inner);

		Self(unsafe { T::from_value_inner(inner) }, Some(gc))
	}

	// safetY: that from_value_inner seems like it could be unsafe potentially lol
	pub fn new_unchecked(t: T) -> Self {
		Self(t, None)
	}

	// SAFETY: caller must ensure that it wno't be GC'd until it's a part of the root node list
	pub unsafe fn assume_used(&self) -> T {
		unsafe { T::from_value_inner(self.0.as_value_inner()) }
	}

	// SAFETY: The return value needs to now reference `self`
	pub unsafe fn with_inner<R>(mut self, func: impl FnOnce(T) -> R) -> R {
		let inner = unsafe { std::ptr::read(&self.0) };
		let result = func(inner);

		self.unroot_inner();
		std::mem::forget(self);
		result
	}

	// Marks the value as a permanent gc root, and returns it.
	pub fn make_permanent(self) -> T {
		let inner = unsafe { std::ptr::read(&self.0) };
		std::mem::forget(self);
		inner
	}

	fn unroot_inner(&mut self) {
		if let Some(gc) = self.1 {
			let mut gc_inner = gc.0.borrow_mut();
			let inner = self.0.as_value_inner();

			if !gc_inner.roots.remove(&inner) {
				unreachable!("unroot of a non-rooted inner? inner={inner:?}, gc={:?}", &gc_inner.roots);
			}
		}
	}

	// pub unsafe fn unroot(mut self) {
	// 	unsafe {
	// 		self.unroot_inner();
	// 	}
	// 	let x = unsafe { std::ptr::read(&self.0) };
	// 	std::mem::forget(self);
	// 	x
	// }
}

impl<T: AsValueInner> Drop for GcRoot<'_, T> {
	fn drop(&mut self) {
		self.unroot_inner();
	}
}

impl<T: AsValueInner> std::ops::Deref for GcRoot<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
