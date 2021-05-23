use std::rc::Rc;
use crate::{Function, Result, Environment, Value};

#[derive(Debug, Clone)]
pub struct Ast(Rc<AstInner>);

#[derive(Debug)]
struct AstInner {
	func: Function,
	args: Box<[Value]>
}

impl Eq for Ast {}
impl PartialEq for Ast {
	fn eq(&self, rhs: &Self) -> bool {
		Rc::ptr_eq(&self.0, &rhs.0)
	}
}

impl Ast {
	pub fn new(func: Function, args: Box<[Value]>) -> Self {
		Self(Rc::new(AstInner { func, args }))
	}

	pub fn run(&self, env: &mut Environment<'_, '_, '_>) -> Result<Value> {
		self.0.func.run(&self.0.args, env)
	}
}