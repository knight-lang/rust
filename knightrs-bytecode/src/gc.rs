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
	pub flags: AtomicU8,
	// TODO: make this data maybeuninit
	pub data: [u8; ALLOC_VALUE_SIZE - std::mem::size_of::<AtomicU8>()],
}

#[repr(C, align(1))]
#[rustfmt::skip]
pub enum Flags {
	GcMarked = 0b0000_0001,
	GcStatic = 0b0000_0010,
	IsString = 0b0000_0100,
	IsList   = 0b0000_1000, // NOTE::: This should only be used if `IsString` has already been checked
	#[cfg(feature = "custom-types")]
	IsCustom = 0b0001_0000,

	Custom1  = 0b0010_0000,
	Custom2  = 0b0100_0000,
	Custom3  = 0b1000_0000,
}

impl Gc {
	pub fn add_root(&mut self, root: Value) {
		self.roots.push(root);
	}

	pub unsafe fn alloc_value_inner(&mut self, flags: u8) -> *mut ValueInner {
		use std::alloc::{alloc, Layout};

		debug_assert_eq!(flags & Flags::GcMarked as u8, 0);
		debug_assert_ne!(
			flags
				& (Flags::IsString as u8
					| Flags::IsList as u8
					| cfg_expr!(feature = "custom-types", Flags::IsCustom as u8, 0)),
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

	pub fn free_value_inner(&mut self, ptr: *mut ValueInner) {
		use std::alloc::{dealloc, Layout};
		unsafe {
			let layout = Layout::new::<ValueInner>();
			dealloc(ptr.cast::<u8>(), layout);
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

pub unsafe trait Sweep {
	// safety: should not be called by anyone other than `gc`
	unsafe fn sweep(self, gc: &mut Gc);
	unsafe fn deallocate(self, gc: &mut Gc);
}

// impl ValueInner {
impl ValueInner {
	fn flags(this: *const Self) -> *const AtomicU8 {
		unsafe { &raw const (*this).flags }
	}

	pub unsafe fn as_knstring(this: *const Self) -> Option<crate::value2::KnString> {
		if unsafe { Self::flags(this).read().load(Ordering::SeqCst) } & Flags::IsString as u8 != 0 {
			Some(unsafe { crate::value2::KnString::from_value_inner(this) })
		} else {
			None
		}
	}

	pub unsafe fn as_list(this: *const Self) -> Option<crate::value2::List> {
		if unsafe { Self::flags(this).read().load(Ordering::SeqCst) }
			& (Flags::IsString as u8 | Flags::IsList as u8)
			== Flags::IsList as u8
		{
			Some(unsafe { crate::value2::List::from_value_inner(this) })
		} else {
			None
		}
	}

	pub unsafe fn mark(this: *const Self) {
		let flags = unsafe { &*Self::flags(this) }.fetch_or(Flags::GcMarked as u8, Ordering::SeqCst);

		if flags
			& (Flags::GcMarked as u8
				| Flags::GcStatic as u8
				| Flags::IsList as u8
				| Flags::IsString as u8)
			== (Flags::GcMarked as u8 | Flags::IsList as u8)
		{
			unsafe {
				Self::as_knstring(this).unwrap_unchecked().mark();
			}
		}
	}

	pub unsafe fn sweep(this: *const Self, gc: &mut Gc) {
		// let old = self.flags_ref().fetch_and(!(Flags::GcMarked as u8), Ordering::SeqCst);

		// if old & Flags::GcMarked as u8 == 0 {
		// 	unsafe {
		// 		self.deallocate(gc);
		// 	}
		// }
		// todo
	}

	pub unsafe fn deallocate(this: *const Self, gc: &mut Gc) {
		// todo
	}
}
// 	unsafe fn mark(&mut self) {
// 		let was_marked = self.flags.fetch_or(Flags::GcMarked as u8, Ordering::SeqCst);
// 		if was_marked & Flags::GcMarked as u8 == 0 {
// 			self as *
// 			// TODO: mark lists
// 		}
// 	}
// }
// #[repr(C)]
// pub struct ValueInner {
// 	_align: ValueAlign,
// 	pub flags: AtomicU8,
// 	pub data: [u8; ALLOC_VALUE_SIZE - std::mem::size_of::<AtomicU8>()],
// }
