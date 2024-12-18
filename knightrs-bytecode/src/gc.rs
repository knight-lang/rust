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
pub enum Flags { // TODO: Don't make this an enum, make it a struct or somethin
	GcIsUsed   = 0b00000_001,
	GcMarked   = 0b00000_010,
	GcStatic   = 0b00000_100,

	IsString   = 0b00001_000,
	IsList     = 0b00010_000,
	#[cfg(feature = "custom-types")]
	IsCustom   = 0b00100_000,
	Custom1    = 0b01000_000,
	Custom2    = 0b10000_000,
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

		if flags & Flags::GcStatic as u8 != 0 {
			return;
		}

		if flags & (Flags::GcMarked as u8 | Flags::IsList as u8 | Flags::IsString as u8)
			== (Flags::GcMarked as u8 | Flags::IsList as u8)
		{
			unsafe {
				Self::as_knstring(this).unwrap_unchecked().mark();
			}
		}
	}

	pub unsafe fn sweep(this: *const Self, gc: &mut Gc) {
		let old =
			unsafe { &*Self::flags(this) }.fetch_and(!(Flags::GcMarked as u8), Ordering::SeqCst);

		if old & Flags::GcStatic as u8 != 0 {
			return;
		}

		if old & (Flags::GcMarked as u8) == 0 {
			unsafe {
				Self::deallocate(this, gc);
			}
		}
	}

	pub unsafe fn deallocate(this: *const Self, gc: &mut Gc) {
		debug_assert_eq!(
			unsafe { &*Self::flags(this) }.load(Ordering::SeqCst) & Flags::GcStatic as u8,
			0
		);

		if let Some(list) = unsafe { Self::as_list(this) } {
			unsafe {
				list.deallocate(gc);
			}
		}

		// TODO: deallocate `*const Self`
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
