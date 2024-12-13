use crate::container::RefCount;
use crate::value2::{Value, ValueRepr};

#[repr(C, align(8))]
pub struct List(Option<RefCount<Inner>>);

// sa::const_assert_eq!(std::mem::align_of::<List>(), super::VALUE_ALLOC_ALIGN);
sa::assert_eq_size!(List, Value);

#[repr(align(16))]
enum Inner {
	Boxed(Value),
	// ...
}

impl List {
	pub(super) fn into_raw(self) -> ValueRepr {
		unsafe { std::mem::transmute(self.0) }
	}

	pub(super) unsafe fn from_raw(repr: ValueRepr) -> Self {
		unsafe { std::mem::transmute(repr) }
	}
}

// #[derive(Debug, Clone)]
// enum ListInner {
// 	Boxed(Box<Value>),
// 	Slice(RefCount<[Value]>),
// 	// Concat(RefCount<[Value]>, RefCount<[Value]>),
// 	Sublist { start: usize, len: usize, slice: RefCount<[Value]> },
// }
