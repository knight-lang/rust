//! The functions within Knight.
use crate::{Value, Result, Error, Number, Environment, Text};
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::convert::{TryInto, TryFrom};
use std::sync::Arc;

type Func = dyn for<'env> Fn(&[Value<'env>], &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>>;

pub struct Function<'func> {
	name: char,
	arity: usize,
	func: &'func Func,
}

impl Debug for Function<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		struct PointerDebug(usize);

		impl Debug for PointerDebug {
			fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
				write!(f, "{:p}", self.0 as *const ())
			}
		}

		if f.alternate() {
			f.debug_struct("Function")
				.field("func", &PointerDebug(self.func_usize()))
				.field("name", &self.name)
				.field("arity", &self.arity)
				.finish()
		} else {
			f.debug_tuple("Function")
				.field(&self.name)
				.finish()
		}
	}
}

impl Eq for Function<'_> {}
impl PartialEq for Function<'_> {
	fn eq(&self, rhs: &Self) -> bool {
		self.name == rhs.name && self.func_usize() == rhs.func_usize()
	}
}

impl Hash for Function<'_> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.func_usize().hash(h);
	}
}

impl<'func> Function<'func> {
	#[inline]
	#[must_use]
	pub fn new(name: char, arity: usize, func: &'func Func) -> Self {
		Self { name, arity, func }
	}

	fn func_usize(&self) -> usize {
		self.func as *const _ as *const () as usize
	}

	#[inline]
	#[must_use]
	pub fn func(&self) -> &Func {
		&*self.func
	}

	#[inline]
	#[must_use]
	pub fn arity(&self) -> usize {
		self.arity
	}

	#[inline]
	#[must_use]
	pub fn name(&self) -> char {
		self.name
	}

	#[inline]
	pub fn run<'env>(&self, args: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
		(self.func)(args, env)
	}
}

use std::io::{Write, BufRead};

// arity zero
pub fn prompt<'env>(_: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
	let mut buf = String::new();

	env.stdin().read_line(&mut buf)?;

	Ok(Text::try_from(buf)?.into())
}

pub fn random<'env>(_: &[Value<'env>], _: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
	unsafe {
		Ok(Number::new_unchecked(rand::random::<u32>() as i64).into())
	}
}

// arity one

pub fn eval<'env>(args: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
	let ran = args[0].run(unsafe { &mut *(env as *mut _) })?;

	env.eval(ran.as_text()?)
}

pub fn block<'env>(args: &[Value<'env>], _: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
	Ok(args[0].clone())
}

// pub fn call<'env>(args: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
// 	args[0].run(env)?.run(env)
// }

// pub fn system<'env>(args: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
// 	let cmd = args[0].run(env)?.to_rcstring()?;

// 	env.system(&cmd).map(Value::from)
// }

// pub fn quit<'env>(args: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
// 	Err(RuntimeError::Quit(args[0].run(env)?.to_number()? as i32))
// }

// pub fn not<'env>(args: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
// 	Ok((!args[0].run(env)?.to_boolean()?).into())
// }

// pub fn length<'env>(args: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
// 	args[0].run(env)?.to_rcstring()
// 		.map(|rcstring| rcstring.len() as Number)
// 		.map(Value::from)
// }

// pub fn dump<'env>(args: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
// 	let ret = args[0].run(env)?;

// 	writeln!(env, "{:?}", ret)?;

// 	Ok(ret)
// }

// pub fn output<'env>(args: &[Value<'env>], env: &'env mut Environment<'_, '_, '_>) -> Result<Value<'env>> {
// 	let text = args[0].run(env)?.to_rcstring()?;

// 	if let Some(stripped) = text.strip_suffix('\\') {
// 		write!(env, "{}", stripped)?;
// 		env.flush()?;
// 	} else {
// 		writeln!(env, "{}", text)?;
// 	}

// 	Ok(Value::default())
// }
