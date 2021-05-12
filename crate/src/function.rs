//! The functions within Knight.

use crate::{Error, Result, Environment};
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::convert::{TryInto, TryFrom};
use std::sync::Mutex;

use crate::text::{ToText, Text};
use crate::boolean::ToBoolean;
use crate::number::{ToNumber, Number, NumberType};
use crate::value::{Value, ValueKind};
use crate::ops::*;

// An alias to make life easier.
type FuncPtr = fn(&[Value], &mut Environment<'_, '_, '_>) -> Result<Value>;

/// The type that represents functions themselves (eg `PROMPT`, `+`, `=`, etc.) within Knight.
/// 
/// Note that [`Function`]s cannot be created directly---you must [`fetch`](Function::fetch) them. New functions can be
/// [`register`](Function::register)ed if so desired.
#[derive(Clone, Copy)]
pub struct Function {
	func: FuncPtr,
	name: char,
	arity: usize
}

impl Debug for Function {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		struct PointerDebug(usize);

		impl Debug for PointerDebug {
			fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
				write!(f, "{:p}", self.0 as *const ())
			}
		}

		if f.alternate() {
			f.debug_struct("Function")
				.field("func", &PointerDebug(self.func as usize))
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

impl Eq for Function {}
impl PartialEq for Function {
	/// Checks to see if two functions are identical.
	///
	/// Two functions are considered the same if their names, arities, and function pointers are identical.
	fn eq(&self, rhs: &Self) -> bool {
		self.name == rhs.name
			&& (self.func as usize) == (rhs.func as usize)
			&& self.arity == rhs.arity
	}
}

impl Hash for Function {
	fn hash<H: Hasher>(&self, h: &mut H) {
		(self.func as usize).hash(h);
	}
}

impl Function {
	/// Gets the function pointer associated with `self`.
	#[inline]
	#[must_use]
	pub fn func(&self) -> FuncPtr {
		self.func
	}

	/// Executes this function with the given arguments
	pub fn run(&self, args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
		(self.func)(args, env)
	}

	/// Gets the arity of this function.
	#[inline]
	#[must_use]
	pub fn arity(&self) -> usize {
		self.arity
	}

	/// Gets the name of the function.
	#[inline]
	#[must_use]
	pub fn name(&self) -> char {
		self.name
	}

	/// Gets the function associate dwith the given `name`, returning `None` if no such function exists.
	#[must_use = "fetching a function does nothing by itself"]
	pub fn fetch(name: char) -> Option<Self> {
		FUNCTIONS.lock().unwrap().get(&name).cloned()
	}

	/// Registers a new function with the given name, discarding any previous value associated with it.
	pub fn register(name: char, arity: usize, func: FuncPtr) {
		FUNCTIONS.lock().unwrap().insert(name, Self { name, arity, func });
	}
}

lazy_static::lazy_static! {
	static ref FUNCTIONS: Mutex<HashMap<char, Function>> = Mutex::new({
		let mut map = HashMap::new();

		macro_rules! insert {
			($name:expr, $arity:expr, $func:expr) => {
				map.insert($name, Function { name: $name, arity: $arity, func: $func });
			};
		}

		// insert!('P', 0, prompt);
		// insert!('R', 0, random);

		// insert!(':', 1, noop);
		// insert!('E', 1, eval);
		// insert!('B', 1, block);
		// insert!('C', 1, call);
		// insert!('`', 1, system);
		// insert!('Q', 1, quit);
		// insert!('!', 1, not);
		// insert!('L', 1, length);
		// insert!('D', 1, dump);
		// insert!('O', 1, output);

		// insert!('+', 2, add);
		// insert!('-', 2, subtract);
		// insert!('*', 2, multiply);
		// insert!('/', 2, divide);
		// insert!('%', 2, modulo);
		// insert!('^', 2, power);
		// insert!('?', 2, equals);
		// insert!('<', 2, less_than);
		// insert!('>', 2, greater_than);
		// insert!('&', 2, and);
		// insert!('|', 2, or);
		// insert!(';', 2, then);
		// insert!('=', 2, assign);
		// insert!('W', 2, r#while);

		// insert!('I', 3, r#if);
		// insert!('G', 3, get);
		// insert!('S', 4, substitute);

		map
	});
}

use std::io::{Write, BufRead};

// arity zero
pub fn prompt(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 0);
	let mut buf = String::new();

	env.read_line(&mut buf)?;

	// remove trailing newlines
	match buf.pop() {
		Some('\n') => match buf.pop() {
			Some('\r') => { /* popped \r\n */ },
			Some(other) => buf.push(other), // ie `?\n`
			None => { /* ie `\n`, so we get empty string */ }
		},
		Some(other) => buf.push(other), // ie didnt end with `\n`
		None => { /* ie buf was empty */ }
	}

	Text::try_from(buf).map(From::from).map_err(From::from)
}

pub fn random(args: &[Value], _: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 0);

	#[cfg(feature = "strict-numbers")] 
	type Num = u16;
	#[cfg(not(feature = "strict-numbers"))] 
	type Num = u32;

	const_assert!((Num::MAX as NumberType) <= Number::MAX.get());
	unsafe {
		Ok(Number::new_unchecked(rand::random::<Num>() as NumberType).into())
	}
}

// // arity one

pub static NOOP_FUNCTION: Function = Function { name: ':', arity: 1, func: noop };
pub fn noop(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);

	args[0].run(env)
}

pub fn eval(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);
	let ran = args[0].run(env)?;

	env.run_str(&ran.to_text()?)
}

pub static BLOCK_FUNCTION: Function = Function { name: 'B', arity: 1, func: block };
pub fn block(args: &[Value], _: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);

	Ok(args[0].clone())
}

pub fn call(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);

	args[0].run(env)?.run(env)
}

pub fn system(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);
	let cmd = args[0].run(env)?;

	env.system(&cmd.to_text()?).map(Value::from)
}

pub fn quit(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);
	let code = args[0].run(env)?.to_number()?.get();

	#[cfg(not(feature = "checked-overflow"))] {
		Err(Error::Quit(code as i32))
	}

	#[cfg(feature = "checked-overflow")] {
		if let Ok(code) = i32::try_from(code) {
			Err(Error::Quit(code))
		} else {
			Err(Error::Domain { message: "exit code is out of range for ran i32" })
		}
	}
}

pub fn not(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);

	Ok((!args[0].run(env)?.to_boolean()?).into())
}

pub fn length(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);
	let arg0 = args[0].run(env)?;
	let text = arg0.to_text()?;

	// The maximum size for a `Text` is `isize::MAX`. So the only way for this to overflow is if we're 64 bit and with
	// strict numbers.

	cfg_if! {
		if #[cfg(all(not(target_pointer_width="32"), feature="strict-numbers", feature="checked-overflow"))] {
			NumberType::try_from(text.len())
				.ok()
				.and_then(Number::new)
				.map(Value::from)
		} else {
			Ok(Number::new_truncate(text.len() as i64).into())
		}
	}
}

pub fn dump(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);
	let ret = args[0].run(env)?;

	writeln!(env, "{:?}", ret)?;

	Ok(ret)
}

pub fn output(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);
	let arg0 = args[0].run(env)?;
	let text = args[0].to_text()?;

	if let Some(stripped) = text.strip_suffix('\\') {
		write!(env, "{}", stripped)?;
		env.flush()?;
	} else {
		writeln!(env, "{}", *text)?;
	}

	Ok(Value::NULL)
}

// arity two

pub fn add(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	args[0].run(env)?.try_add(&args[1].run(env)?)
}

pub fn subtract(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	args[0].run(env)?.try_sub(&args[1].run(env)?)
}

pub fn multiply(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	args[0].run(env)?.try_mul(&args[1].run(env)?)
}

pub fn divide(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	args[0].run(env)?.try_div(&args[1].run(env)?)
}

pub fn modulo(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	args[0].run(env)?.try_rem(&args[1].run(env)?)
}

pub fn exponentiate(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	args[0].run(env)?.try_pow(&args[1].run(env)?)
}

pub fn equals(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	Ok((args[0].run(env)? == args[1].run(env)?).into())
}
// pub fn equals(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 2);

// 	Ok((args[0].run(env)? == args[1].run(env)?).into())
// }

// pub fn less_than(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 2);

// 	match args[0].run(env)? {
// 		Value::Number(lhs) => Ok((lhs < args[1].run(env)?.to_number()?).into()),
// 		Value::Boolean(lhs) => Ok((lhs < args[1].run(env)?.to_boolean()?).into()),
// 		Value::Text(lhs) => Ok((lhs.as_str() < args[1].run(env)?.to_text()?.as_str()).into()),
// 		other => Err(Error::Type { func: '<', operand: other.typename() })
// 	}
// }

// pub fn greater_than(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 2);

// 	match args[0].run(env)? {
// 		Value::Number(lhs) => Ok((lhs > args[1].run(env)?.to_number()?).into()),
// 		Value::Boolean(lhs) => Ok((lhs > args[1].run(env)?.to_boolean()?).into()),
// 		Value::Text(lhs) => Ok((lhs.as_str() > args[1].run(env)?.to_text()?.as_str()).into()),
// 		other => Err(Error::Type { func: '>', operand: other.typename() })
// 	}
// }

// pub fn and(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 2);

// 	let lhs = args[0].run(env)?;

// 	if lhs.to_boolean()? {
// 		args[1].run(env)
// 	} else {
// 		Ok(lhs)
// 	}
// }

// pub fn or(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 2);

// 	let lhs = args[0].run(env)?;

// 	if lhs.to_boolean()? {
// 		Ok(lhs)
// 	} else {
// 		args[1].run(env)
// 	}
// }

// pub fn then(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 2);

// 	args[0].run(env)?;
// 	args[1].run(env)
// }

// pub fn assign(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 2);

// 	let variable = 
// 		if let Value::Variable(ref variable) = args[0] {
// 			variable
// 		} else /* if cfg!(feature = "assign-to-anything") */ {
// 			return Err(Error::Type { func: '?', operand: args[0].typename() });
// 		};

// 	let rhs = args[1].run(env)?;

// 	variable.assign(rhs.clone());

// 	Ok(rhs)
// }

// pub fn r#while(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 2);

// 	while args[0].run(env)?.to_boolean()? {
// 		let _ = args[1].run(env)?;
// 	}

// 	Ok(Value::default())
// }

// // arity three

// pub fn r#if(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 3);

// 	if args[0].run(env)?.to_boolean()? {
// 		args[1].run(env)
// 	} else {
// 		args[2].run(env)
// 	}
// }

// pub fn get(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 3);

// 	let input = args[0].run(env)?.to_text()?;
// 	let start = args[1].run(env)?.to_number()?;
// 	let len = args[2].run(env)?.to_number()?;

// 	let start = start as usize; // todo: check
// 	let len = len as usize; // todo: check

// 	Ok(Value::Text(Text::new(&input[start..start+len]).unwrap()))
// }

// // arity four

// pub fn substitute(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
// 	debug_assert_eq!(args.len(), 4);

// 	let source = args[0].run(env)?.to_text()?;
// 	let start = args[1].run(env)?.to_number()?;
// 	let len = args[2].run(env)?.to_number()?;
// 	let repl = args[3].run(env)?.to_text()?;

// 	let start = start as usize; // todo: check
// 	let len = len as usize; // todo: check

// 	if start == 0 && repl.len() == 0 {
// 		return Ok(Value::Text(Text::new(&source[len..]).unwrap()));
// 	}

// 	let mut result = String::with_capacity(source.len() - len + repl.len());

// 	result.push_str(&source[..start]);
// 	result.push_str(&repl);
// 	result.push_str(&source[start+len..]);

// 	Ok(Value::Text(result.try_into().unwrap())) // we know the replacement is valid, as both sources were valid.
// }
