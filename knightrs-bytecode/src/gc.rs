use std::sync::atomic::{AtomicU8, Ordering};

use crate::value2::{Value, ValueAlign};

#[derive(Default)]
pub struct Gc {
	value_inners: Vec<*mut ValueInner>,
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

pub const FLAG_GC_MARKED: u8 = 1 << 0;
pub const FLAG_GC_STATIC: u8 = 1 << 1;
pub const FLAG_IS_STRING: u8 = 1 << 2;
pub const FLAG_IS_LIST: u8 = 1 << 3;
#[cfg(feature = "custom-types")]
pub const FLAG_IS_CUSTOM: u8 = 1 << 4; // must check if `FLAG_IS_STRING` isn't set,as it uses FLAG_CUSTOM_0_DONTUSE
pub const FLAG_CUSTOM_0_DONTUSE: u8 = 1 << 4;

pub const FLAG_CUSTOM1: u8 = 1 << 5;
pub const FLAG_CUSTOM2: u8 = 1 << 6;
pub const FLAG_CUSTOM3: u8 = 1 << 7;

impl Gc {
	pub fn add_root(&mut self, root: Value) {
		self.roots.push(root);
	}

	pub unsafe fn alloc_value_inner(&mut self, flags: u8) -> *mut ValueInner {
		use std::alloc::{alloc, Layout};

		debug_assert_eq!(flags & FLAG_GC_MARKED, 0, "cannot already be marked");
		debug_assert_ne!(
			flags
				& (FLAG_IS_STRING
					| FLAG_IS_LIST
					| cfg_expr!(feature = "custom-types", FLAG_IS_CUSTOM, 0)),
			0,
			"need a type passed in"
		);

		unsafe {
			let inner = alloc(Layout::new::<ValueInner>()).cast::<ValueInner>();
			if inner.is_null() {
				panic!("alloc failed");
			}

			(&raw mut (*inner).flags).write(AtomicU8::new(flags));

			self.value_inners.push(inner);
			inner
		}
	}

	pub fn mark_and_sweep(&mut self) {
		assert_ne!(self.roots.len(), 0, "called mark_and_sweep during mark and sweep");

		for root in &mut self.roots {
			unsafe {
				root.mark();
			}
		}

		// TODO: we should be sweeping not from roots but for _all_ values
		let mut roots = std::mem::take(&mut self.roots);
		for root in &mut roots {
			unsafe {
				root.sweep(self);
			}
		}
		self.roots = roots;
	}

	pub unsafe fn shutdown(&mut self) {
		for root in std::mem::take(&mut self.roots) {
			unsafe {
				root.deallocate(self);
			}
		}
	}
}

// safety: has to make sure there's no cycle. shouldn't be for any builtin types.
pub unsafe trait Mark {
	// safety: should not be called by anyone other than `gc`
	unsafe fn mark(&mut self);
}

pub trait Allocated {
	// safety: caller ensures `this` is the only reference
	unsafe fn deallocate(self, gc: &mut Gc);
}

pub unsafe trait Sweep {
	// safety: should not be called by anyone other than `gc`
	unsafe fn sweep(self, gc: &mut Gc);
	unsafe fn deallocate(self, gc: &mut Gc);
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

	pub unsafe fn sweep(this: *const Self, gc: &mut Gc) {
		let old = unsafe { &*Self::flags(this) }.fetch_and(!FLAG_GC_MARKED, Ordering::SeqCst);

		// If it's static then just return
		if old & FLAG_GC_STATIC != 0 {
			return;
		}

		// If it's not marked, ie `mark` didn't mark it, then deallocate it.
		if old & FLAG_GC_MARKED == 0 {
			unsafe {
				Self::deallocate(this, gc);
			}
		}
	}

	pub unsafe fn deallocate(this: *const Self, gc: &mut Gc) {
		debug_assert_eq!(unsafe { &*Self::flags(this) }.load(Ordering::SeqCst) & FLAG_GC_STATIC, 0);

		if let Some(string) = unsafe { Self::as_knstring(this) } {
			unsafe {
				string.deallocate(gc);
			}
		} else if let Some(list) = unsafe { Self::as_list(this) } {
			unsafe {
				list.deallocate(gc);
			}
		} else {
			unreachable!("non-list non-string encountered?");
		}

		// Mark it as `0` to indicate it's unused.
		unsafe { &*Self::flags(this) }.store(0, Ordering::SeqCst);
	}
}
