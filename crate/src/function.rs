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

		#[cfg(feature = "eval-function")]
		_ if name == EVAL.name => Some(&EVAL),

		#[cfg(feature = "arrays")]
		_ if name == BOX.name => Some(&BOX),

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
pub const VALUE: Function = function!('V', env, |arg| {
	let name = arg.run(env)?.to_knstr()?;
	env.lookup(&name)
});

/// **4.2.3** `BLOCK`  
pub const BLOCK: Function = function!('B', env, |arg| {
	#[cfg(feature = "strict-block-return-value")]
	if !matches!(arg, Value::Ast(_)) {
		const NOOP: Function = function!(':', _, |arg| {
			debug_assert!(!matches!(arg, Value::Ast(_)));

			arg.clone()
		});

		return Ok(crate::Ast::new(&NOOP, vec![arg.clone()]).into());
	}

	arg.clone()
});

/// **4.2.4** `CALL`  
pub const CALL: Function = function!('C', env, |arg| {
	let block = arg.run(env)?;

	#[cfg(feature = "strict-block-return-value")]
	if !matches!(block, Value::Ast(_)) {
		return Err(Error::TypeError("only blocks may be executed via `CALL`."));
	}

	block.run(env)?
});

/// **4.2.5** `` ` ``  
pub const SYSTEM: Function = function!('`', env, |arg| {
	let command = arg.run(env)?.to_knstr()?;

	env.run_command(&command)?
});

/// **4.2.6** `QUIT`  
pub const QUIT: Function = function!('Q', env, |arg| {
	return arg
		.run(env)?
		.to_integer()?
		.try_conv::<i32>()
		.map(|status| Err(Error::Quit(status)))
		.or(Err(Error::DomainError("exit code out of bounds")))?;

	// The `function!` macro calls `.into()` on the return value, so we need _something_ here.
	#[allow(dead_code)]
	Value::Null
});

/// **4.2.7** `!`  
pub const NOT: Function = function!('!', env, |arg| !arg.run(env)?.to_bool()?);

/// **4.2.8** `LENGTH`  
pub const LENGTH: Function = function!('L', env, |arg| arg.run(env)?.to_knstr()?.len() as Integer);

/// **4.2.9** `DUMP`  
pub const DUMP: Function = function!('D', env, |arg| {
	let value = arg.run(env)?;
	writeln!(env, "{value:?}")?;
	value
});

/// **4.2.10** `OUTPUT`  
pub const OUTPUT: Function = function!('O', env, |arg| {
	let text = arg.run(env)?.to_knstr()?;

	if text.ends_with('\\') {
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
	let ran = arg.run(env)?;

	#[cfg(feature = "arrays")]
	if let Value::Array(ary) = ran {
		let mut copy = ary.iter().collect::<Vec<Value>>();
		copy.reverse();
		return Ok(crate::Array::from(copy).into());
	}

	let num = ran.to_integer()?;

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
		Value::SharedStr(string) => string.concat(&rhs.run(env)?.to_knstr()?).conv::<Value>(),
		#[cfg(feature = "arrays")]
		Value::Array(lary) => {
			let rary = rhs.run(env)?.to_array()?;
			let mut cat = Vec::with_capacity(lary.len() + rary.len());
			cat.extend(lary.iter());
			cat.extend(rary.iter());
			crate::Array::from(cat).into()
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
				lnum.checked_sub(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				lnum.wrapping_sub(rnum).conv::<Value>()
			}
		}

		#[cfg(feature = "arrays")]
		Value::Array(lary) => {
			let rary = rhs.run(env)?.to_array()?;
			let mut array = Vec::with_capacity(lary.len());

			for ele in &*lary.as_slice() {
				if !rary.contains(ele) && !array.contains(ele) {
					array.push(ele.clone());
				}
			}

			crate::Array::from(array).into()
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
			let amount = rhs
				.run(env)?
				.to_integer()?
				.try_conv::<usize>()
				.or(Err(Error::DomainError("repetition count is negative")))?;

			if isize::MAX as usize <= amount * lstr.len() {
				return Err(Error::DomainError("repetition is too large"));
			}

			lstr.repeat(amount).conv::<Value>()
		}

		#[cfg(feature = "arrays")]
		Value::Array(lary) => match rhs.run(env)? {
			Value::Boolean(true) => lary.into(),
			Value::Boolean(false) => crate::Array::default().into(),
			Value::Integer(amount @ 0..) => {
				let mut array = Vec::with_capacity(lary.len() * (amount as usize));

				for _ in 0..amount {
					array.extend(lary.iter());
				}

				crate::Array::from(array).into()
			}
			Value::SharedStr(string) => {
				let mut joined = String::new();

				let mut is_first = true;
				for ele in &*lary.as_slice() {
					if is_first {
						is_first = false;
					} else {
						joined.push_str(&string);
					}

					joined.push_str(&ele.to_knstr()?);
				}

				SharedStr::try_from(joined).unwrap().into()
			}
			Value::Array(rary) => {
				let mut result = Vec::with_capacity(lary.len() * rary.len());

				for lele in lary.iter() {
					for rele in rary.iter() {
						result.push(crate::Array::from(vec![lele.clone(), rele.clone()]).into());
					}
				}

				crate::Array::from(result).into()
			}
			Value::Ast(ast) => {
				let mut result = Vec::with_capacity(lary.len());
				let arg = env.lookup("_".try_into().unwrap());

				for ele in lary.iter() {
					arg.assign(ele.clone());
					result.push(ast.run(env)?);
				}

				crate::Array::from(result).into()
			}
			other => return Err(Error::TypeError(other.typename())),
		},

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
				lnum.checked_div(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				lnum.wrapping_div(rnum).conv::<Value>()
			}
		}

		#[cfg(feature = "split-strings")]
		Value::SharedStr(lstr) => {
			let rstr = rhs.run(env)?.to_knstr()?;

			if rstr.is_empty() {
				return Ok(Value::SharedStr(lstr).to_array()?.into());
			}

			lstr
				.split(&**rstr)
				.map(|x| SharedStr::new(x).unwrap().into())
				.collect::<crate::Array>()
				.into()
		}

		#[cfg(feature = "arrays")]
		Value::Array(lary) => match rhs.run(env)? {
			Value::Ast(ast) => {
				let acc_var = env.lookup("a".try_into().unwrap());

				if let Some(init) = lary.as_slice().get(0) {
					acc_var.assign(init.clone());
				}

				let arg_var = env.lookup("_".try_into().unwrap());

				for ele in lary.iter().skip(1) {
					arg_var.assign(ele);
					acc_var.assign(ast.run(env)?);
				}

				acc_var.fetch().unwrap_or_default()
			}
			other => return Err(Error::TypeError(other.typename())),
		},

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
				lnum.checked_rem(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
			}

			#[cfg(not(feature = "checked-overflow"))]
			{
				lnum.wrapping_rem(rnum).conv::<Value>()
			}
		}

		#[cfg(feature = "string-formatting")]
		Value::SharedStr(lstr) => {
			let values = rhs.run(env)?.to_array()?;
			let mut values_index = 0;

			let mut formatted = String::new();
			let mut chars = lstr.chars();

			while let Some(chr) = chars.next() {
				match chr {
					'\\' => {
						formatted.push(match chars.next().expect("<todo error for nothing next>") {
							'n' => '\n',
							'r' => '\r',
							't' => '\t',
							'{' => '{',
							'}' => '}',
							_ => panic!("todo: error for unknown escape code"),
						});
					}
					'{' => {
						if chars.next() != Some('}') {
							panic!("todo, missing closing `}}`");
						}
						formatted.push_str(
							&values
								.as_slice()
								.get(values_index)
								.expect("no values left to format")
								.to_knstr()?,
						);
						values_index += 1;
					}
					_ => formatted.push(chr),
				}
			}

			SharedStr::new(formatted).unwrap().into()
		}

		#[cfg(feature = "arrays")]
		Value::Array(lary) => match rhs.run(env)? {
			Value::Ast(ast) => {
				let mut result = Vec::new();
				let arg_var = env.lookup("_".try_into().unwrap());

				for ele in lary.iter() {
					arg_var.assign(ele.clone());
					if ast.run(env)?.to_bool()? {
						result.push(ele);
					}
				}

				crate::Array::from(result).into()
			}
			other => return Err(Error::TypeError(other.typename())),
		},
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
			if base == 1 {
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
					_ => lnum.wrapping_pow(rnum as u32),
				}
			}
			.conv::<Value>()
		}
		#[cfg(feature = "arrays")]
		Value::Array(lary) => {
			let max = rhs.run(env)?.to_integer()?;
			assert!(max >= 0, "todo, negative amounts");
			(0..max).map(Value::from).collect::<crate::Array>().into()
		}

		other => return Err(Error::TypeError(other.typename())),
	}
});

fn compare(lhs: Value, rhs: Value) -> Result<std::cmp::Ordering> {
	match lhs {
		Value::Integer(lnum) => Ok(lnum.cmp(&rhs.to_integer()?)),
		Value::Boolean(lbool) => Ok(lbool.cmp(&rhs.to_bool()?)),
		Value::SharedStr(ltext) => Ok(ltext.cmp(&rhs.to_knstr()?)),
		#[cfg(feature = "arrays")]
		Value::Array(lary) => {
			let rary = rhs.to_array()?;
			for (lele, rele) in lary.iter().zip(rary.iter()) {
				match compare(lele, rele)? {
					std::cmp::Ordering::Equal => {}
					other => return Ok(other),
				}
			}

			Ok(lary.len().cmp(&rary.len()))
		}
		other => Err(Error::TypeError(other.typename())),
	}
}

/// **4.3.7** `<`  
pub const LESS_THAN: Function = function!('<', env, |lhs, rhs| {
	compare(lhs.run(env)?, rhs.run(env)?)? == std::cmp::Ordering::Less
});

/// **4.3.8** `>`  
pub const GREATER_THAN: Function = function!('>', env, |lhs, rhs| {
	compare(lhs.run(env)?, rhs.run(env)?)? == std::cmp::Ordering::Greater
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
fn assign(variable: &Value, value: Value, env: &mut Environment) -> Result<()> {
	match variable {
		Value::Variable(var) => {
			var.assign(value);
		}

		#[cfg(feature = "arrays")]
		Value::Array(ary) => {
			if ary.is_empty() {
				panic!("todo: error for this case");
			}
			let rhs = value.run(env)?.to_array()?;

			for (name, val) in ary.as_slice().iter().zip(rhs.iter()) {
				assign(name, val, env)?;
			}

			match ary.len().cmp(&rhs.len()) {
				std::cmp::Ordering::Equal => {}
				std::cmp::Ordering::Less => assign(
					ary.as_slice().iter().last().unwrap(),
					rhs.as_slice()[ary.len() - 1..].iter().cloned().collect::<crate::Array>().into(),
					env,
				)?,
				std::cmp::Ordering::Greater => {
					for extra in &ary.as_slice()[rhs.len()..] {
						assign(extra, Value::default(), env)?;
					}
				}
			}
		}

		#[cfg(feature = "arrays")]
		Value::Ast(ast) => return assign(&variable.run(env)?, value, env),

		_ => {
			let name = variable.run(env)?.to_knstr()?;
			env.lookup(&name).assign(value);
		}
	}

	Ok(())
}

pub const ASSIGN: Function = function!('=', env, |var, value| {
	let ret = value.run(env)?;
	assign(var, ret.clone(), env)?;
	ret
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
	let source = string.run(env)?;
	let start = start
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative start position")))?;
	let length = length
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative length")))?;

	#[cfg(feature = "arrays")]
	if let Value::Array(ary) = source {
		return Ok(if length == 0 {
			ary.as_slice().get(start).unwrap().clone()
		} else {
			ary.as_slice()
				.get(start..start + length)
				.expect("Todo: error")
				.iter()
				.cloned()
				.collect::<crate::Array>()
				.into()
		});
	}

	let string = source.to_knstr()?;
	match string.get(start..start + length) {
		Some(value) => value.to_boxed().conv::<SharedStr>(),

		#[cfg(feature = "out-of-bounds-errors")]
		None => return Err(Error::IndexOutOfBounds { len: string.len(), index: start + length }),

		#[cfg(not(feature = "out-of-bounds-errors"))]
		None => SharedStr::default(),
	}
});

/// **4.5.1** `SUBSTITUTE`  
pub const SUBSTITUTE: Function = function!('S', env, |string, start, length, replacement| {
	let source = string.run(env)?;
	let start = start
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative start position")))?;
	let length = length
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative length")))?;
	let replacement_source = replacement.run(env)?;

	#[cfg(feature = "arrays")]
	if let Value::Array(ary) = source {
		if length == 0 {
			let mut dup = ary.iter().collect::<Vec<Value>>();
			dup[start] = replacement_source.into();
			return Ok(crate::Array::from(dup).into());
		}

		let replacement = replacement_source.to_array()?;

		let mut ret = Vec::new();
		ret.extend(ary.iter().take(start));
		ret.extend(replacement.iter());
		ret.extend(ary.iter().skip(start + length));

		return Ok(crate::Array::from(ret).into());
	}

	let string = source.to_knstr()?;
	let replacement = replacement_source.to_knstr()?;
	// TODO: `out-of-bounds-errors` here
	// lol, todo, optimize me
	let mut s = String::new();
	s.push_str(&string[..start]);
	s.push_str(&replacement);
	s.push_str(&string[start + length..]);
	s.try_conv::<SharedStr>().unwrap()
});

/// EXT: Eval
#[cfg(feature = "eval-function")]
pub const EVAL: Function = function!('E', env, |val| {
	let code = val.run(env)?.to_knstr()?;
	env.play(&code)?
});

/// EXT: Box
#[cfg(feature = "arrays")]
pub const BOX: Function = function!(',', env, |val| crate::Array::from(vec![val.run(env)?]));
