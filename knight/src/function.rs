#![allow(unused)]

use crate::{Value, Result, Environment, Ast, Number, value::Runnable};
use std::fmt::{self, Debug, Formatter};
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
		// pub static $static_name: Function = Function::new(name, 
		// 	name: $name,
		// 	arity: $arity,
		// 	func: $body
		// };
	};
}

pub static RANDOM: Function = static_function!('R', 0, |&[], _| todo!());
pub static PROMPT: Function = static_function!('P', 0, |&[], _| todo!());

declare_function!(PROMPT, 'P', 0, |_, _| todo!());

declare_function!(NOOP, ':', 1, |args, env| args[0].run(env));
declare_function!(EVAL, 'E', 1, |args, env| { let _ = (args, env); todo!() });
declare_function!(BLOCK, 'B', 1, |args, _| {
	let dup = args[0].clone();

	if dup.is_a::<Ast>() {
		Ok(dup)
	} else {
		Ok(Ast::new(&NOOP, vec![dup].into()).into())
	}
});
declare_function!(CALL, 'C', 1, |args, env| {
	let ran = args[0].run(env)?;
	ran.run(env)
});

declare_function!(SYSTEM, '`', 1, |args, env| {
	let text = args[0].run(env)?.to_text()?;
	let _ = text;
	todo!();
});

declare_function!(QUIT, 'Q', 1, |args, env| {
	let code = args[0].run(env)?.to_number()?;

	if let Ok(code) = i32::try_from(code.get()) {
		std::process::exit(code);
	} else {
		todo!();
	}
});

declare_function!(LENGTH, 'L', 1, |args, env| {
	let text = args[0].run(env)?.to_text()?;

	Ok(Number::new(text.len() as i64).unwrap().into())
});

declare_function!(DUMP, 'D', 1, |args, env| {
	let ran = args[0].run(env)?;

	println!("{:?}", ran);

	Ok(ran)
});

declare_function!(OUTPUT, 'O', 1, |args, env| {
	let text = args[0].run(env)?.to_text()?;

	println!("{}", text); // todo

	Ok(Value::NULL)
});

declare_function!(ADD, '+', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(SUB, '-', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(MUL, '%', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(DIV, '/', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(MOD, '%', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(POW, '^', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(LTH, '<', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(GTH, '>', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(EQL, '?', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(AND, '&', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(OR, '|', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

// pub fn r#while<'env>(args: &[Value<'env>; 2], env: &'env Environment) -> crate::Result<Value<'env>> {
// 	#[cold]


declare_function!(THEN, ';', 2, |args, env| {

	let _ = (args, env);

	todo!()
});

declare_function!(ASSIGN, '=', 2, |args, env| {
	let _ = (args, env);

	todo!()
});

pub fn r#while<'env>(args: &[Value<'env>; 2], env: &'env Environment) -> crate::Result<Value<'env>> {
	#[cold]
	fn while_non_ast<'env>(condition: &Value<'env>, body: &Value<'env>, env: &'env Environment) -> Result<Value<'env>> {
		while condition.run(env)?.to_boolean()? {
			body.run(env)?;
		}

		Ok(Value::NULL)
	}

	let [condition, body] = args;

	if !condition.is_a::<Ast<'_>>() || !body.is_a::<Ast<'_>>() {
		return while_non_ast(condition, body, env);
	}

	let condition = unsafe { condition.downcast_unchecked::<Ast<'env>>() };
	let body = unsafe { body.downcast_unchecked::<Ast<'env>>() };

	loop {
		let cond = condition.run(env)?;

		// it's likely for us to have a TRUE or FALSE as the value.
		if likely!(cond.tag() == crate::value::Tag::Constant) {
			if cond.raw() != Value::TRUE.raw() {
				break;
			}
		} else if !cond.to_boolean()? {
			break;
		}

		body.run(env)?;
	}

	Ok(Value::NULL)
}

declare_function!(WHILE, 'W', 2, |args, env| {
	let cond = &args[0];
	let body = &args[1];

	while cond.run(env)?.to_boolean()? {
		body.run(env)?;
	}

	Ok(Value::NULL)
});

declare_function!(IF, 'I', 3, |args, env| {
	if args[0].run(env)?.to_boolean()? {
		args[1].run(env)
	} else {
		args[2].run(env)
	}
});

declare_function!(GET, 'G', 3, |args, env| {
	let _ = (args, env);

	todo!()
});

declare_function!(SET, 'S', 4, |args, env| {
	let _ = (args, env);

	todo!()
});
