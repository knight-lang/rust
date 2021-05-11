use crate::{Function, Result, Value, Environment};
use std::rc::Rc;

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

	#[inline]
	pub fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Value> {
		self.0.func.run(&self.0.args, env)
	}

	pub(crate) fn into_raw(self) -> *const () {
		Rc::into_raw(self.0) as *const ()
	}

	pub(crate) unsafe fn from_raw(raw: *const ()) -> Self {
		Self(Rc::from_raw(raw as *const Inner))
	}

	pub(crate) unsafe fn clone_in_place(raw: *const ()) {
		let this = Self::from_raw(raw);
		std::mem::forget(this.clone()); // add one to the refcount.
		std::mem::forget(this);         // make sure we don't drop this reference.
	}

	pub(crate) unsafe fn drop_in_place(raw: *const ()) {
		drop(Self::from_raw(raw));
	}
}