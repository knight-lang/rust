use crate::Function;
use crate::value2::Value;
use std::rc::Rc;
use std::num::NonZeroU64;

#[derive(Debug, Clone)]
pub struct Ast(Rc<Inner>);

#[derive(Debug)]
struct Inner {
	func: Function,
	args: Box<[Value]>
}

impl Ast {
	pub fn new(func: Function, args: Box<[Value]>) -> Self {
		assert_eq!(func.arity(), args.len());

		Self(Rc::new(Inner { func, args }))
	}

	pub(crate) fn into_raw(self) -> NonZeroU64 {
		unsafe {
			NonZeroU64::new_unchecked(Rc::into_raw(self.0) as usize as u64)
		}
	}

	pub(crate) unsafe fn from_raw(raw: NonZeroU64) -> Self {
		Self(Rc::from_raw(raw.get() as usize as *const _))
	}
}