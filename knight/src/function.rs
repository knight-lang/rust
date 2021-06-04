#![allow(unused)]

use crate::{Value, Result, Environment, Ast, Number, Error, Variable};
use std::fmt::{self, Debug, Formatter};
use crate::ops::*;
use std::convert::TryFrom;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Function {
	arity: usize,
	trampoline: for<'env> unsafe fn(unsafe fn(), *const Value<'env>, &'env Environment) -> Result<Value<'env>>,
	func: unsafe fn(), // actually a fn(&[Value; arity], &Environment) -> Result<Value>
	name: char,
}

#[repr(C)]
#[doc(hidden)]
pub struct _StaticFunctionBuilder {
	pub arity: usize,
	pub trampoline: for<'env> unsafe fn(unsafe fn(), *const Value<'env>, &'env Environment) -> Result<Value<'env>>,
	pub func: unsafe fn(), // actually a fn(&[Value; arity], &Environment) -> Result<Value>
	pub name: char,
}

impl Debug for Function {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		struct PtrDisp(usize);
		impl Debug for PtrDisp {
			fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
				write!(f, "{:p}", self.0 as *const ())
			}
		}

		if f.alternate() {
			f.debug_struct("Function")
				.field("name", &self.name)
				.field("arity", &self.arity)
				.field("func", &PtrDisp(self.func as usize))
				.finish()
		} else {
			f.debug_tuple("Function")
				.field(&self.name)
				.finish()
		}
	}
}

#[macro_export]
macro_rules! static_function {
	($name:literal, $arity:literal, $function:expr) => {unsafe {
		use std::mem::transmute;

		transmute::<$crate::function::_StaticFunctionBuilder, $crate::Function>($crate::function::_StaticFunctionBuilder {
			name: $name,
			arity: $arity,
			trampoline: |f, val, env| unsafe {

				let f = transmute::<
					_,
					for<'e> fn(&[Value<'e>; $arity], &'e Environment) -> Result<Value<'e>>
				>(f);

				f(transmute::<*const Value<'_>, &[Value<'_>; $arity]>(val), env)
			},
			func: unsafe {
				transmute::<
					for<'e> fn(&[Value<'e>; $arity], &'e Environment) -> Result<Value<'e>>,
					unsafe fn()
				>($function)
			}
		})
	}}
}

impl Function {
	// pub unsafe fn _new<const ARITY: usize>(
	// 	name: char,
	// 	trampoline: for<'env> unsafe fn(unsafe fn(), *const Value<'env>, &'env Environment) -> Result<Value<'env>>,
	// 	func: unsafe fn() // actually a fn(&[Value; arity], &Environment) -> Result<Value>
	// ) -> Self {
	// 	Self { name, trampoline, func, arity: ARITY }
	// }

	pub const fn arity(&self) -> usize {
		self.arity
	}

	pub fn run<'env>(&self, args: &[Value<'env>], env: &'env  Environment) -> Result<Value<'env>> {
		debug_assert_eq!(self.arity(), args.len());

		unsafe {
			(self.trampoline)(self.func, args.as_ptr(), env)
		}
	}
}

macro_rules! declare_function {
	($static_name:ident, $name:literal, $arity:literal, $body:expr) => {
		pub static $static_name: Function = static_function!($name, $arity, $body);
	};
}

pub static PROMPT: Function = static_function!('P', 0, |[], _| todo!());
pub static RANDOM: Function = static_function!('R', 0, |[], _| todo!());

pub static NOOP: Function = static_function!(':', 1, |[arg], env| {
	arg.run(env)
});

pub static EVAL: Function = static_function!('E', 1, |[text], env| {
	let text = text.run(env)?.to_text();

	let _ = env;
	todo!();
});

pub static BLOCK: Function = static_function!('B', 1, |[block], _| {
	let dup = block.clone();

	if dup.is_a::<Ast>() {
		return Ok(dup);
	}

	Ok(Ast::new(&NOOP, vec![dup].into()).into())
});

pub static CALL: Function = static_function!('C', 1, |[block], env| {
	if let Some(ast) = block.downcast::<Ast>() {
		let ran = ast.run(env)?;
		ran.run(env)
	} else {
		Err(Error::InvalidArgument { func: 'C', kind: block.typename() })
	}
});

pub static SYSTEM: Function = static_function!('`', 1, |[command], env| {
	let command = command.run(env)?.to_text()?;

	Ok(env.system(command.as_str())?.into())
});

pub static QUIT: Function = static_function!('Q', 1, |[code], env| {
	let code = code.run(env)?.to_number()?;

	if let Ok(code) = i32::try_from(code.get()) {
		std::process::exit(code);
	} else {
		todo!();
	}
});

pub static LENGTH: Function = static_function!('L', 1, |[text], env| {
	let text = text.run(env)?.to_text()?;

	Ok(Number::new(text.len() as i64).unwrap().into())
});

pub static DUMP: Function = static_function!('D', 1, |[arg], env| {
	let ran = arg.run(env)?;

	println!("{:?}", ran);

	Ok(ran)
});

pub static OUTPUT: Function = static_function!('O', 1, |[arg], env| {
	let text = arg.run(env)?.to_text()?;

	println!("{}", text); // todo

	Ok(Value::NULL)
});

pub static ADD: Function = static_function!('+', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;
	let rhs = rhs.run(env)?;

	lhs.try_add(rhs)
});

pub static SUB: Function = static_function!('-', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;
	let rhs = rhs.run(env)?;

	lhs.try_sub(rhs)
});

pub static MUL: Function = static_function!('*', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;
	let rhs = rhs.run(env)?;

	lhs.try_mul(rhs)
});

pub static DIV: Function = static_function!('/', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;
	let rhs = rhs.run(env)?;

	lhs.try_div(rhs)
});

pub static MOD: Function = static_function!('%', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;
	let rhs = rhs.run(env)?;

	lhs.try_rem(rhs)
});

pub static POW: Function = static_function!('^', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;
	let rhs = rhs.run(env)?;

	lhs.try_pow(rhs)
});

pub static LTH: Function = static_function!('<', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;
	let rhs = rhs.run(env)?;

	match lhs.try_cmp(&rhs) {
		Err(Error::InvalidArgument { kind, func: 'c' }) => Err(Error::InvalidArgument { kind, func: '<' }),
		Err(other) => Err(other),
		Ok(std::cmp::Ordering::Less) => Ok(Value::TRUE),
		Ok(_)  => Ok(Value::FALSE)
	}
});

pub static GTH: Function = static_function!('>', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;
	let rhs = rhs.run(env)?;

	match lhs.try_cmp(&rhs) {
		Err(Error::InvalidArgument { kind, func: 'c' }) => Err(Error::InvalidArgument { kind, func: '>' }),
		Err(other) => Err(other),
		Ok(std::cmp::Ordering::Greater) => Ok(Value::TRUE),
		Ok(_)  => Ok(Value::FALSE)
	}
});

pub static EQL: Function = static_function!('?', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;
	let rhs = rhs.run(env)?;

	Ok(lhs.try_eq(&rhs)?.into())
});

pub static AND: Function = static_function!('&', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;

	if lhs.to_boolean()? {
		rhs.run(env)
	} else {
		Ok(lhs)
	}
});

pub static OR: Function = static_function!('|', 2, |[lhs, rhs], env| {
	let lhs = lhs.run(env)?;

	if lhs.to_boolean()? {
		Ok(lhs)
	} else {
		rhs.run(env)
	}
});


pub static THEN: Function = static_function!(';', 2, |[lhs, rhs], env| {
	lhs.run(env)?;
	rhs.run(env)
});

pub static ASSIGN: Function = static_function!('=', 2, |[variable, value], env| {
	let value = value.run(env)?;

	if let Some(variable) = variable.downcast::<Variable>() {
		variable.set(value.clone());
		Ok(value)
	} else {
		Err(Error::InvalidArgument { func: '=', kind: variable.typename() })
	}
});

pub static WHILE: Function = static_function!('W', 2, |[cond, body], env| {
	while cond.run(env)?.to_boolean()? {
		body.run(env)?;
	}

	Ok(Value::NULL)
});

pub static IF: Function = static_function!('I', 3, |[cond, if_true, if_false], env| {
	if cond.run(env)?.to_boolean()? {
		if_true.run(env)
	} else {
		if_false.run(env)
	}
});

pub static GET: Function = static_function!('G', 3, |[text, start, length], env| {
	let text = text.run(env)?.to_text()?;
	let start = start.run(env)?.to_number()?;
	let length = length.run(env)?.to_number()?;

	let _ = (text, start, length);
	todo!()
});

pub static SET: Function = static_function!('S', 4, |[text, start, length, replacement], env| {
	let text = text.run(env)?.to_text()?;
	let start = start.run(env)?.to_number()?;
	let length = length.run(env)?.to_number()?;
	let replacement = replacement.run(env)?.to_text()?;

	let _ = (text, start, length, replacement);
	todo!()
});
