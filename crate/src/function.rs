use crate::{Environment, Error, Integer, Result, SharedStr, Value};
use std::fmt::{self, Debug, Formatter};
use std::io::{BufRead, Write};
use tap::prelude::*;

#[derive(Clone, Copy)]
pub struct Function {
	/// The code associated with this function
	pub func: fn(&[Value], &mut Environment) -> Result<Value>,

	/// The single character name of this function
	pub name: char,

	/// The arity of the function.
	pub arity: usize,
}

// impl Function {
// 	pub const fn new(func: )
// }

impl Eq for Function {}
impl PartialEq for Function {
	fn eq(&self, rhs: &Self) -> bool {
		std::ptr::eq(self, rhs)
	}
}

impl Debug for Function {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Function({})", self.name)
	}
}

pub const fn fetch(name: char) -> Option<&'static Function> {
	match name {
		_ if name == PROMPT.name => Some(&PROMPT),
		_ if name == RANDOM.name => Some(&RANDOM),
		#[cfg(not(feature = "no-extensions"))]
		_ if name == EVAL.name => Some(&EVAL),
		_ if name == VALUE.name => Some(&VALUE),
		_ if name == BLOCK.name => Some(&BLOCK),
		_ if name == CALL.name => Some(&CALL),
		_ if name == SYSTEM.name => Some(&SYSTEM),
		_ if name == QUIT.name => Some(&QUIT),
		_ if name == NOT.name => Some(&NOT),
		_ if name == LENGTH.name => Some(&LENGTH),
		_ if name == DUMP.name => Some(&DUMP),
		_ if name == OUTPUT.name => Some(&OUTPUT),
		_ if name == ASCII.name => Some(&ASCII),
		_ if name == NEG.name => Some(&NEG),
		_ if name == ADD.name => Some(&ADD),
		_ if name == SUBTRACT.name => Some(&SUBTRACT),
		_ if name == MULTIPLY.name => Some(&MULTIPLY),
		_ if name == DIVIDE.name => Some(&DIVIDE),
		_ if name == MODULO.name => Some(&MODULO),
		_ if name == POWER.name => Some(&POWER),
		_ if name == EQUALS.name => Some(&EQUALS),
		_ if name == LESS_THAN.name => Some(&LESS_THAN),
		_ if name == GREATER_THAN.name => Some(&GREATER_THAN),
		_ if name == AND.name => Some(&AND),
		_ if name == OR.name => Some(&OR),
		_ if name == THEN.name => Some(&THEN),
		_ if name == ASSIGN.name => Some(&ASSIGN),
		_ if name == WHILE.name => Some(&WHILE),
		_ if name == IF.name => Some(&IF),
		_ if name == GET.name => Some(&GET),
		_ if name == SUBSTITUTE.name => Some(&SUBSTITUTE),
		_ => None,
	}
}

macro_rules! arity {
	() => (0);
	($_pat:ident $($rest:ident)*) => (1+arity!($($rest)*))
}
macro_rules! function {
	($name:literal, $env:pat, |$($args:ident),*| $body:expr) => {
		Function {
			name: $name,
			arity: arity!($($args)*),
			func: |args, $env| {
				let [$($args,)*]: &[Value; arity!($($args)*)] = args.try_into().unwrap();
				Ok($body.into())
			}
		}
	};
}

/// **4.1.4**: `PROMPT`
pub const PROMPT: Function = function!('P', env, |/*.*/| {
	let mut buf = String::new();

	env.read_line(&mut buf)?;

	// remove trailing newlines
	match buf.pop() {
		Some('\n') => match buf.pop() {
			Some('\r') => {}
			Some(other) => buf.push(other), // ie `<anything>\n`
			None => {}
		},
		Some(other) => buf.push(other),
		None => {}
	}

	SharedStr::try_from(buf)?
});

/// **4.1.5**: `RANDOM`
pub const RANDOM: Function = function!('R', env, |/*.*/| env.random());

/// **4.2.2** `VALUE`  
pub const VALUE: Function =
	function!('V', e, |name| name.run(e)?.to_knstr()?.pipe(|name| e.lookup(&name)));

/// **4.2.3** `BLOCK`  
pub const BLOCK: Function = function!('B', env, |arg| {
	#[cfg(feature = "strict-block-return-value")]
	{
		const NOOP: Function = function!(':', _, |arg| {
			debug_assert!(!matches!(arg, Value::Ast(_)));

			arg.clone()
		});

		if !matches!(arg, Value::Ast(_)) {
			return Ok(crate::Ast::new(&NOOP, vec![arg.clone()]).into());
		}
	}

	arg.clone()
});

/// **4.2.4** `CALL`  
pub const CALL: Function = function!('C', env, |arg| {
	let torun = arg.run(env)?;

	if cfg!(feature = "strict-block-return-value") && !matches!(torun, Value::Ast(_)) {
		return Err(Error::TypeError("only blocks may be executed via `CALL`."));
	}

	torun.run(env)?.run(env)?
});

/// **6.6** `EVAL`
#[cfg(not(feature = "no-extensions"))]
pub const EVAL: Function = function!('E', env, |arg| {
	let input = arg.run(env)?.to_knstr()?;
	crate::Parser::new(&input).parse(env)?.run(env)?
});

/// **4.2.5** `` ` ``  
pub const SYSTEM: Function = function!('`', env, |arg| {
	let command = arg.run(env)?.to_knstr()?;
	env.run_command(&command)?
});

/// **4.2.6** `QUIT`  
pub const QUIT: Function = function!('Q', env, |arg| {
	let status = arg
		.run(env)?
		.to_integer()?
		.try_conv::<i32>()
		.or(Err(Error::DomainError("exit code out of bounds")))?;

	return Err(Error::Quit(status));

	#[allow(dead_code)]
	Value::Null
});

/// **4.2.7** `!`  
pub const NOT: Function = function!('!', env, |arg| { (!arg.run(env)?.to_bool()?) });

/// **4.2.8** `LENGTH`  
pub const LENGTH: Function =
	function!('L', env, |arg| { (arg.run(env)?.to_knstr()?.len() as Integer) });

/// **4.2.9** `DUMP`  
pub const DUMP: Function = function!('D', env, |arg| {
	let value = arg.run(env)?;
	writeln!(env, "{value:?}")?;
	value
});

/// **4.2.10** `OUTPUT`  
pub const OUTPUT: Function = function!('O', env, |arg| {
	let text = arg.run(env)?.to_knstr()?;

	if text.chars().last() == Some('\\') {
		write!(env, "{}", &text[..text.len() - 1])?
	} else {
		writeln!(env, "{text}")?;
	}

	Value::Null
});

/// **4.2.11** `ASCII`  
pub const ASCII: Function = function!('A', env, |arg| {
	match arg.run(env)? {
		Value::Integer(num) => u32::try_from(num)
			.ok()
			.and_then(char::from_u32)
			.and_then(|chr| SharedStr::new(chr).ok())
			.ok_or(Error::DomainError("number isn't a valid char"))?
			.conv::<Value>(),

		Value::SharedStr(text) => text
			.chars()
			.next()
			.ok_or(Error::DomainError("empty string"))?
			.pipe(|x| x as Integer)
			.conv::<Value>(),

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.2.12** `~`  
pub const NEG: Function = function!('~', env, |arg| {
	let num = arg.run(env)?.to_integer()?;

	#[cfg(feature = "checked-overflow")]
	{
		num.checked_neg().ok_or(Error::IntegerOverflow)?
	}

	#[cfg(not(feature = "checked-overflow"))]
	{
		num.wrapping_neg()
	}
});

/// **4.3.1** `+`  
pub const ADD: Function = function!('+', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			#[cfg(feature = "checked-overflow")]
			{
				lnum.checked_add(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				lnum.wrapping_add(rnum).conv::<Value>()
			}
		}
		Value::SharedStr(lstr) => {
			let rstr = rhs.run(env)?.to_knstr()?;
			let mut cat = String::with_capacity(lstr.len() + rstr.len());
			cat.push_str(&lstr);
			cat.push_str(&rstr);

			SharedStr::try_from(cat).unwrap().conv::<Value>()
		}

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.2** `-`  
pub const SUBTRACT: Function = function!('-', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			#[cfg(feature = "checked-overflow")]
			{
				lnum.checked_sub(rnum).ok_or(Error::IntegerOverflow)?
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				lnum.wrapping_sub(rnum)
			}
		}
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.3** `*`  
pub const MULTIPLY: Function = function!('*', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			#[cfg(feature = "checked-overflow")]
			{
				lnum.checked_mul(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				lnum.wrapping_mul(rnum).conv::<Value>()
			}
		}
		Value::SharedStr(lstr) => {
			// clean me up
			let amount = rhs
				.run(env)?
				.to_integer()?
				.try_conv::<usize>()
				.map_err(|_| Error::DomainError("repetition length not within bounds"))?;

			if amount * lstr.len() >= (isize::MAX as usize) {
				return Err(Error::DomainError("repetition length not within bounds"));
			}

			lstr.repeat(amount).try_conv::<SharedStr>().unwrap().conv::<Value>()
		}
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.4** `/`  
pub const DIVIDE: Function = function!('/', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			if rnum == 0 {
				return Err(Error::DivisionByZero);
			}

			#[cfg(feature = "checked-overflow")]
			{
				lnum.checked_div(rnum).ok_or(Error::IntegerOverflow)?
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				lnum.wrapping_div(rnum)
			}
		}
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.5** `%`  
pub const MODULO: Function = function!('%', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			if rnum == 0 {
				return Err(Error::DivisionByZero);
			}

			#[cfg(feature = "checked-overflow")]
			{
				lnum.checked_rem(rnum).ok_or(Error::IntegerOverflow)?
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				lnum.wrapping_rem(rnum)
			}
		}
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.6** `^`  
pub const POWER: Function = function!('^', env, |lhs, rhs| {
	// TODO: verify this is actually power work
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;
			let base = lnum;
			let exponent = rnum;

			// TODO: clean me up
			(if base == 1 {
				1
			} else if base == -1 {
				if exponent & 1 == 1 {
					-1
				} else {
					1
				}
			} else {
				match exponent {
					1 => base,
					0 => 1,
					_ if base == 0 && exponent < 0 => return Err(Error::DivisionByZero),
					_ if exponent < 0 => 0,
					#[cfg(feature = "checked-overflow")]
					_ => lnum.checked_pow(rnum as u32).ok_or(Error::IntegerOverflow)?,
					#[cfg(not(feature = "checked-overflow"))]
					_ => lnum.wrapping_pow(rnum as u32).into(),
				}
			})
		}
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.7** `<`  
pub const LESS_THAN: Function = function!('<', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => lnum < rhs.run(env)?.to_integer()?,
		Value::Boolean(lbool) => !lbool & rhs.run(env)?.to_bool()?,
		Value::SharedStr(ltext) => ltext < rhs.run(env)?.to_knstr()?,
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.8** `>`  
pub const GREATER_THAN: Function = function!('>', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => lnum > rhs.run(env)?.to_integer()?,
		Value::Boolean(lbool) => lbool & !rhs.run(env)?.to_bool()?,
		Value::SharedStr(ltext) => ltext > rhs.run(env)?.to_knstr()?,
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.9** `?`  
pub const EQUALS: Function = function!('?', env, |lhs, rhs| {
	let l = lhs.run(env)?;
	let r = rhs.run(env)?;

	if cfg!(feature = "strict-compliance") {
		if matches!(l, Value::Ast(_) | Value::Variable(_)) {
			return Err(Error::TypeError(l.typename()));
		}

		if matches!(r, Value::Ast(_) | Value::Variable(_)) {
			return Err(Error::TypeError(r.typename()));
		}
	}

	l == r
});

/// **4.3.10** `&`  
pub const AND: Function = function!('&', env, |lhs, rhs| {
	let l = lhs.run(env)?;

	if l.to_bool()? {
		rhs.run(env)?
	} else {
		l
	}
});

/// **4.3.11** `|`  
pub const OR: Function = function!('|', env, |lhs, rhs| {
	let l = lhs.run(env)?;

	if l.to_bool()? {
		l
	} else {
		rhs.run(env)?
	}
});

/// **4.3.12** `;`  
pub const THEN: Function = function!(';', env, |lhs, rhs| {
	lhs.run(env)?;
	rhs.run(env)?
});

/// **4.3.13** `=`  
pub const ASSIGN: Function = function!('=', env, |var, value| {
	let variable = if let Value::Variable(v) = var {
		v.clone()
	} else {
		let name = var.run(env)?.to_knstr()?;
		env.lookup(&name)
	};

	let ran = value.run(env)?;
	variable.assign(ran.clone());
	ran
});

/// **4.3.14** `WHILE`  
pub const WHILE: Function = function!('W', env, |cond, body| {
	while cond.run(env)?.to_bool()? {
		body.run(env)?;
	}

	Value::Null
});

/// **4.4.1** `IF`  
pub const IF: Function = function!('I', env, |cond, iftrue, iffalse| {
	if cond.run(env)?.to_bool()? {
		iftrue.run(env)?
	} else {
		iffalse.run(env)?
	}
});

/// **4.4.2** `GET`  
pub const GET: Function = function!('G', env, |string, start, length| {
	let string = string.run(env)?.to_knstr()?;
	let start = start.run(env)?.to_integer()?.try_conv::<usize>().expect("todo");
	let length = length.run(env)?.to_integer()?.try_conv::<usize>().expect("todo");

	// lol, todo, optimize me
	string
		.get(start..start + length)
		.expect("todo: error for out of bounds")
		.to_boxed()
		.conv::<SharedStr>()
});

/// **4.5.1** `SUBSTITUTE`  
pub const SUBSTITUTE: Function = function!('S', env, |string, start, length, replacement| {
	let string = string.run(env)?.to_knstr()?;
	let start = start.run(env)?.to_integer()?.try_conv::<usize>().expect("todo");
	let length = length.run(env)?.to_integer()?.try_conv::<usize>().expect("todo");
	let replacement = replacement.run(env)?.to_knstr()?;

	// lol, todo, optimize me
	let mut s = String::new();
	s.push_str(&string[..start]);
	s.push_str(&replacement);
	s.push_str(&string[start + length..]);
	s.try_conv::<SharedStr>().unwrap()
});
