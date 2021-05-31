use crate::Variable;
use std::collections::HashSet;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

#[derive(Default)]
pub struct Environment {
	vars: HashSet<VariableHash<'static>> // actually 'self, as we know the pointer's always valid.
}

impl Drop for Environment {
	fn drop(&mut self) {
		// todo!("drop vars")
	}
}

impl Environment {
	pub fn fetch_var<'env, N: ?Sized>(&'env mut self, name: &N) -> Variable<'env>
	where
		N: Borrow<str> + ToString
	{
		if let Some(VariableHash(var)) = self.vars.get(name.borrow()) {
			// This is ok because the variable will only last as long as `self`'s reference will, and will be thrown away
			// when it's done.
			unsafe {
				std::mem::transmute::<Variable<'static>, Variable<'env>>(*var)
			}
		} else {
			let var = Variable::new(name.to_string().into_boxed_str());

			self.vars.insert(VariableHash(unsafe {
				std::mem::transmute::<Variable<'env>, Variable<'static>>(var)
			}));

			var
		}
	}
}

struct VariableHash<'env>(Variable<'env>);

impl Drop for VariableHash<'_> {
	fn drop(&mut self) {
		unsafe {
			self.0.drop_in_place();
		}
	}
}

impl Hash for VariableHash<'_> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.0.name().hash(h)
	}
}

impl Borrow<str> for VariableHash<'_> {
	fn borrow(&self) -> &str {
		self.0.name()
	}
}

impl Eq for VariableHash<'_> {}
impl PartialEq<str> for VariableHash<'_> {
	fn eq(&self, rhs: &str) -> bool {
		self.0.name() == rhs
	}
}

impl PartialEq for VariableHash<'_> {
	fn eq(&self, rhs: &Self) -> bool {
		(self.0) == (rhs.0)
	}
}
