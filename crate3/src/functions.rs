//! The functions within Knight.
use crate::{Value, ValueKind, Result, Error, Number, Environment, Text, Boolean, Null};
use crate::text::TextBuilder;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::convert::{TryInto, TryFrom};
use std::sync::Arc;

type Func = dyn Fn(&[Value], &mut Environment<'_, '_, '_>) -> Result<Value>;

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
	pub fn run(&self, args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
		(self.func)(args, env)
	}
}

use std::io::{Write, BufRead};

// arity zero
pub fn prompt(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 0);

	let mut buf = String::new();

	env.read_line(&mut buf)?;

	Ok(Text::try_from(buf)?.into())
}

pub fn random(args: &[Value], _: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 0);

	unsafe {
		Ok(Number::new_unchecked(rand::random::<u32>() as i64).into())
	}
}

// arity one

pub fn eval(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);

	let ran = args[0].run(env)?;
	env.eval(ran.to_text()?)
}

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
	Ok(env.system(&cmd.to_text()?)?.into())
}

pub fn quit(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);

	let code = args[0].run(env)?.to_number()?.inner();

	if let Ok(code) = i32::try_from(code) {
		Err(Error::Quit { code })
	} else {
		Err(Error::LengthOverflow { message: "too large of a code given" })
	}
}

pub fn not(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);

	let arg = args[0].run(env)?.to_boolean()?;

	Ok((!arg).into())
}

pub fn length(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 1);

	let text = args[0].run(env)?;

	if let Some(number) = Number::new(text.to_text()?.len() as i64) {
		Ok(number.into())
	} else {
		Err(Error::LengthOverflow { message: "text length is too large to represent?" })
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

	let text = args[0].run(env)?;
	let text = text.to_text()?;

	if let Some(stripped) = text.strip_suffix('\\') {
		write!(env, "{}", stripped)?;
		env.flush()?;
	} else {
		writeln!(env, "{}", text)?;
	}

	Ok(Null.into())
}


// arity two
pub fn add(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;
	let rhs = args[1].run(env)?;

	match lhs.classify() {
		ValueKind::Number(lhs) => {
			let rhs = rhs.to_number()?;
			let sum = lhs.inner() + rhs.inner(); // adding two `Number`s will never overflow `i64`.

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					Number::new(sum)
						.map(Value::from)
						.ok_or_else(|| Error::Overflow { func: '+', lhs, rhs })
				} else {
					Ok(Number::new_truncate(sum).into())
				}
			}
		},
		ValueKind::Text(lhs) => {
			let rhs = args[1].run(env)?.to_text()?;

			// both values are valid texts, so adding them also yields a valid text
			Ok(Text::new_owned(lhs.to_string() + &rhs).unwrap().into())
		},
		other => Err(Error::InvalidOperand { func: '+', operand: other.typename() })
	}
}

pub fn subtract(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;
	let rhs = args[1].run(env)?;

	match lhs.classify() {
		ValueKind::Number(lhs) => {
			let rhs = rhs.to_number()?;
			let difference = lhs.inner() - rhs.inner();

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					Number::new(difference)
						.map(Value::from)
						.ok_or_else(|| Error::Overflow { func: '-', lhs, rhs })
				} else {
					Ok(Number::new_truncate(difference).into())
				}
			}
		},
		other => Err(Error::InvalidOperand { func: '-', operand: other.typename() })
	}
}

pub fn multiply(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;
	let rhs = args[1].run(env)?;

	match lhs.classify() {
		ValueKind::Number(lhs) => {
			let rhs = rhs.to_number()?;

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lhs.inner().checked_mul(rhs.inner())
						.and_then(Number::new)
						.map(Value::from)
						.ok_or_else(|| Error::Overflow { func: '*', lhs, rhs })
				} else {
					Ok(Number::new_truncate(lhs.inner() * rhs.inner()).into())
				}
			}
		}
		ValueKind::Text(lhs) => {
			let count =
				args[1]
					.to_number()?
					.inner()
					.try_into()
					.or(Err(Error::BadArgument { func: '*', reason: "negative count given", }))?;

			let mut builder = TextBuilder::with_capacity(lhs.len() * count);

			for i in 0..count {
				builder.append(&lhs);
			}

			Ok(builder.build()?.into())
		},
		other => Err(Error::InvalidOperand { func: '*', operand: other.typename() })
	}
}

pub fn divide(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;
	let rhs = args[1].run(env)?;

	match lhs.classify() {
		ValueKind::Number(lhs) => {
			let rhs = rhs.to_number()?;

			let quotient = 
				lhs.inner().checked_div(rhs.inner())
					.ok_or(Error::BadArgument { func: '/', reason: "division by zero" })?;

			// Number::MIN / -1 will be unrepresentable, so we have to when enabled.
			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					Number::new(quotient)
						.map(Value::from)
						.ok_or_else(|| Error::Overflow { func: '/', lhs, rhs })
				} else {
					Ok(Number::new_truncate(quotient).into())
				}
			}
		},
		other => Err(Error::InvalidOperand { func: '/', operand: other.typename() })
	}
}

pub fn modulo(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;
	let rhs = args[1].run(env)?;

	match lhs.classify() {
		ValueKind::Number(lhs) => {
			let rhs = rhs.to_number()?;

			if lhs.inner() < 0 || rhs.inner() < 0 {
				Err(Error::BadArgument { func: '%', reason: "modulo with negative values" })
			} else if rhs.inner() == 0 {
				Err(Error::BadArgument { func: '%', reason: "modulo by zero" })
			} else {
				Ok(Number::new(lhs.inner() % rhs.inner()).expect("modulo cant overflow?").into())
			}
		}
		other => Err(Error::InvalidOperand { func: '%', operand: other.typename() })
	}
}


// TODO: checked-overflow for this function.
pub fn power(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;
	let rhs = args[1].run(env)?;

	match lhs.classify() {
		ValueKind::Number(lhs) => {
			let rhs = rhs.to_number()?;

			if lhs.inner() == 0 && rhs.inner() < 0 {
				return Err(Error::BadArgument { func: '^', reason: "exponentiate by zero" })
			}

			// In Knight, all numbers are maximally reqired to be i32. For all `i32 ^ i32` with a
			// result representable by an `i32`, the result is the same as converting them to f64, then powf.
			let result = (lhs.inner() as f64).powf(rhs.inner() as f64);

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					if result as i64 as f64 == result {
						Number::new(result as i64).map(Value::from)
					} else {
						None
					}.ok_or_else(|| Error::Overflow { func: '^', lhs, rhs })
				} else {
					Ok(Number::new_truncate(result as i64).into())
				}
			}
		},
		other => Err(Error::InvalidOperand { func: '^', operand: other.typename() })
	}
}

pub fn equals(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;
	let rhs = args[1].run(env)?;

	Ok((lhs == rhs).into())
}

pub fn less_than(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;
	let rhs = args[1].run(env)?;

	match lhs.classify() {
		ValueKind::Number(lhs) => Ok((lhs < rhs.to_number()?).into()),
		ValueKind::Boolean(lhs) => Ok((lhs < rhs.to_boolean()?).into()),
		ValueKind::Text(lhs) => Ok((*lhs < rhs.to_text()?).into()),
		other => Err(Error::InvalidOperand { func: '<', operand: other.typename() })
	}
}

pub fn greater_than(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;
	let rhs = args[1].run(env)?;

	match lhs.classify() {
		ValueKind::Number(lhs) => Ok((lhs > rhs.to_number()?).into()),
		ValueKind::Boolean(lhs) => Ok((lhs > rhs.to_boolean()?).into()),
		ValueKind::Text(lhs) => Ok((*lhs > rhs.to_text()?).into()),
		other => Err(Error::InvalidOperand { func: '>', operand: other.typename() })
	}
}

pub fn and(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;

	if lhs.to_boolean()? {
		args[1].run(env)
	} else {
		Ok(lhs)
	}
}



pub fn or(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let lhs = args[0].run(env)?;

	if lhs.to_boolean()? {
		Ok(lhs)
	} else {
		args[1].run(env)
	}
}

pub fn then(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	args[0].run(env)?;
	args[1].run(env)
}

pub fn assign(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	let variable = 
		if let Some(variable) = args[0].as_variable()  {
			variable
		} else if cfg!(feature = "assign-to-anything") {
			let lhs = args[0].run(env)?;
			env.variable(&lhs.to_text()?)
		} else {
			return Err(Error::InvalidOperand { func: '=', operand: args[0].typename() })
		};

	let rhs = args[1].run(env)?;
	variable.set_value(rhs.clone());

	Ok(rhs)
}

pub fn r#while(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 2);

	while args[0].run(env)?.to_boolean()? {
		let _ = args[1].run(env)?;
	}

	Ok(Null.into())
}

// arity three

pub fn r#if(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 3);

	if args[0].run(env)?.to_boolean()? {
		args[1].run(env)
	} else {
		args[2].run(env)
	}
}

fn calculate_start_stop(text_len: usize, start: i64, len: i64, func: char) -> Result<(usize, usize)> {
	let start =
		if start < 0 {
			if cfg!(feature = "negative-indexing") {
				text_len - (start.abs() as usize)
			} else {
				return Err(Error::BadArgument { func, reason: "negative indexing is not enabled" });
			}
		} else {
			start as usize
		};

	let stop =
		if len < 0 {
			return Err(Error::BadArgument { func, reason: "cannot get negative length substring" });
		} else if text_len < (len as usize) + start { // allow for being equal to text len.
			return Err(Error::BadArgument { func, reason: "substring length is past end of string" });
		} else {
			(len as usize) + start
		};

	Ok((start, stop))

}

pub fn get(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 3);

	let source = args[0].run(env)?.to_text()?;
	let start = args[1].run(env)?.to_number()?.inner();
	let len = args[2].run(env)?.to_number()?.inner();

	let (start, stop) = calculate_start_stop(source.len(), start, len, 'G')?;

	Ok(Text::new_borrowed(&source.as_str()[start..stop])
		.unwrap()
		.into())
}


// arity four

pub fn substitute(args: &[Value], env: &mut Environment<'_, '_, '_>) -> Result<Value> {
	debug_assert_eq!(args.len(), 4);

	let source = args[0].run(env)?.to_text()?;
	let start = args[1].run(env)?.to_number()?.inner();
	let len = args[2].run(env)?.to_number()?.inner();
	let replacement = args[3].run(env)?.to_text()?;

	let (start, stop) = calculate_start_stop(source.len(), start, len, 'S')?;

	let mut builder = TextBuilder::with_capacity(source.len() + (stop - start) + replacement.len());

	builder.append(&source[..start]);
	builder.append(&replacement);
	builder.append(&source[stop..]);

	Ok(builder.build().unwrap().into())
}
