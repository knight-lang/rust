use crate::{Environment, Error, Integer, List, Result, SharedText, Value};
use std::fmt::{self, Debug, Formatter};
use std::io::{BufRead, Write};
use tap::prelude::*;

/// A function in knight indicates
#[derive(Clone, Copy)]
pub struct Function {
	/// The code associated with this function
	pub func: fn(&[Value], &mut Environment) -> Result<Value>,

	/// The short-form name of this function.
	///
	/// For extension functions that start with `X`, this should always be `X`.
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
		_ if name == EVAL.name => Some(&EVAL),
		_ if name == BLOCK.name => Some(&BLOCK),
		_ if name == CALL.name => Some(&CALL),
		_ if name == SYSTEM.name => Some(&SYSTEM),
		_ if name == QUIT.name => Some(&QUIT),
		_ if name == NOT.name => Some(&NOT),
		_ if name == LENGTH.name => Some(&LENGTH),
		_ if name == DUMP.name => Some(&DUMP),
		_ if name == OUTPUT.name => Some(&OUTPUT),
		_ if name == ASCII.name => Some(&ASCII),
		_ if name == BOX.name => Some(&BOX),
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
		_ if name == SET.name => Some(&SET),

		#[cfg(feature = "value-function")]
		_ if name == VALUE.name => Some(&VALUE),

		#[cfg(feature = "handle-function")]
		_ if name == HANDLE.name => Some(&HANDLE),

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
	#[cfg(feature="assign-to-prompt")]
	if let Some(line) = env.get_next_prompt_line() {
		return Ok(line.into());
	}

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

	buf.try_conv::<SharedText>()?
});

/// **4.1.5**: `RANDOM`
pub const RANDOM: Function = function!('R', env, |/*.*/| env.random());

/// **4.2.2** `EVAL`  
pub const EVAL: Function = function!('E', env, |val| {
	let code = val.run(env)?.to_text()?;
	env.play(&code)?
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
		return Err(Error::TypeError(block.typename()));
	}

	block.run(env)?
});

/// **4.2.5** `` ` ``  
pub const SYSTEM: Function = function!('`', env, |arg| {
	let command = arg.run(env)?.to_text()?;

	env.run_command(&command)?
});

/// **4.2.6** `QUIT`  
pub const QUIT: Function = function!('Q', env, |arg| {
	let status = arg
		.run(env)?
		.to_integer()?
		.try_conv::<i32>()
		.or(Err(Error::DomainError("exit code out of bounds")))?;

	if cfg!(feature = "strict-compliance") && !(0..=127).contains(&status) {
		return Err(Error::DomainError("exit code out of bounds"));
	}

	return Err(Error::Quit(status));

	// The `function!` macro calls `.into()` on the return value of this block,
	// , so we need _something_ here so it can typecheck correctly.
	#[allow(dead_code)]
	Value::Null
});

/// **4.2.7** `!`  
pub const NOT: Function = function!('!', env, |arg| !arg.run(env)?.to_bool()?);

/// **4.2.8** `LENGTH`  
pub const LENGTH: Function = function!('L', env, |arg| arg.run(env)?.to_text()?.len() as Integer);

/// **4.2.9** `DUMP`  
pub const DUMP: Function = function!('D', env, |arg| {
	let value = arg.run(env)?;
	writeln!(env, "{value:?}")?;
	value
});

/// **4.2.10** `OUTPUT`  
pub const OUTPUT: Function = function!('O', env, |arg| {
	let text = arg.run(env)?.to_text()?;

	if let Some(stripped) = text.strip_suffix('\\') {
		write!(env, "{stripped}")?
	} else {
		writeln!(env, "{text}")?;
	}

	env.flush()?;

	Value::Null
});

/// **4.2.11** `ASCII`  
pub const ASCII: Function = function!('A', env, |arg| {
	match arg.run(env)? {
		Value::Integer(num) => u32::try_from(num)
			.ok()
			.and_then(char::from_u32)
			.and_then(|chr| SharedText::new(chr).ok())
			.ok_or(Error::DomainError("number isn't a valid char"))?
			.conv::<Value>(),

		Value::SharedText(text) => text
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
	if let Value::List(list) = ran {
		let mut copy = list.iter().cloned().collect::<Vec<Value>>();
		copy.reverse();
		return Ok(List::from(copy).into());
	}

	let num = ran.to_integer()?;

	cfg_if! {
		if #[cfg(feature = "checked-overflow")] {
			num.checked_neg().ok_or(Error::IntegerOverflow)?
		} else {
			num.wrapping_neg()
		}
	}
});

/// EXT: Box
pub const BOX: Function = function!(',', env, |val| List::from(vec![val.run(env)?]));

/// **4.3.1** `+`  
pub const ADD: Function = function!('+', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_add(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_add(rnum).conv::<Value>()
				}
			}
		}

		Value::SharedText(string) => string.concat(&rhs.run(env)?.to_text()?).conv::<Value>(),

		#[cfg(feature = "arrays")]
		Value::List(llist) => {
			let rlist = rhs.run(env)?.to_array()?;
			let mut cat = Vec::with_capacity(llist.len() + rlist.len());
			cat.extend(llist.iter().cloned());
			cat.extend(rlist.iter().cloned());
			List::from(cat).into()
		}
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.2** `-`  
pub const SUBTRACT: Function = function!('-', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_sub(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_sub(rnum).conv::<Value>()
				}
			}
		}

		#[cfg(feature = "arrays")]
		Value::List(llist) => {
			let rlist = rhs.run(env)?.to_array()?;
			let mut list = Vec::with_capacity(llist.len());

			for ele in &*llist.as_slice() {
				if !rlist.contains(ele) && !list.contains(ele) {
					list.push(ele.clone());
				}
			}

			List::from(list).into()
		}

		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.3** `*`  
pub const MULTIPLY: Function = function!('*', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(lnum) => {
			let rnum = rhs.run(env)?.to_integer()?;

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_mul(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_mul(rnum).conv::<Value>()
				}
			}
		}
		Value::SharedText(lstr) => {
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
		Value::List(llist) => match rhs.run(env)? {
			Value::Boolean(true) => llist.into(),
			Value::Boolean(false) => List::default().into(),
			Value::Integer(amount @ 0..) => {
				let mut list = Vec::with_capacity(llist.len() * (amount as usize));

				for _ in 0..amount {
					list.extend(llist.iter().cloned());
				}

				List::from(list).into()
			}
			Value::SharedText(string) => {
				let mut joined = String::new();

				let mut is_first = true;
				for ele in &*llist.as_slice() {
					if is_first {
						is_first = false;
					} else {
						joined.push_str(&string);
					}

					joined.push_str(&ele.to_text()?);
				}

				SharedText::try_from(joined).unwrap().into()
			}
			Value::List(rlist) => {
				let mut result = Vec::with_capacity(llist.len() * rlist.len());

				for lele in llist.iter() {
					for rele in rlist.iter() {
						result.push(List::from(vec![lele.clone(), rele.clone()]).into());
					}
				}

				List::from(result).into()
			}
			Value::Ast(ast) => {
				let mut result = Vec::with_capacity(llist.len());
				let arg = env.lookup("_".try_into().unwrap())?;

				for ele in llist.iter() {
					arg.assign(ele.clone());
					result.push(ast.run(env)?);
				}

				List::from(result).into()
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

			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_div(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_div(rnum).conv::<Value>()
				}
			}
		}

		#[cfg(feature = "split-strings")]
		Value::SharedText(lstr) => {
			let rstr = rhs.run(env)?.to_text()?;

			if rstr.is_empty() {
				return Ok(Value::SharedText(lstr).to_array()?.into());
			}

			lstr.split(&**rstr).map(|x| SharedText::new(x).unwrap().into()).collect::<List>().into()
		}

		#[cfg(feature = "arrays")]
		Value::List(llist) => match rhs.run(env)? {
			Value::Ast(ast) => {
				let acc_var = env.lookup("a".try_into().unwrap())?;

				if let Some(init) = llist.as_slice().get(0) {
					acc_var.assign(init.clone());
				}

				let arg_var = env.lookup("_".try_into().unwrap())?;

				for ele in llist.iter().skip(1) {
					arg_var.assign(ele.clone());
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

			if cfg!(feature = "strict-compliance") && rnum < 0 {
				return Err(Error::DomainError("modulo by a negative base"));
			}

			// TODO: check if `rem` actually follows the specs.
			cfg_if! {
				if #[cfg(feature = "checked-overflow")] {
					lnum.checked_rem(rnum).ok_or(Error::IntegerOverflow)?.conv::<Value>()
				} else {
					lnum.wrapping_rem(rnum).conv::<Value>()
				}
			}
		}

		#[cfg(feature = "string-formatting")]
		Value::SharedText(lstr) => {
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
								.to_text()?,
						);
						values_index += 1;
					}
					_ => formatted.push(chr),
				}
			}

			SharedText::new(formatted).unwrap().into()
		}

		#[cfg(feature = "arrays")]
		Value::List(llist) => match rhs.run(env)? {
			Value::Ast(ast) => {
				let mut result = Vec::new();
				let arg_var = env.lookup("_".try_into().unwrap())?;

				for ele in llist.iter() {
					arg_var.assign(ele.clone());

					if ast.run(env)?.to_bool()? {
						result.push(ele.clone());
					}
				}

				List::from(result).into()
			}
			other => return Err(Error::TypeError(other.typename())),
		},
		other => return Err(Error::TypeError(other.typename())),
	}
});

/// **4.3.6** `^`  
pub const POWER: Function = function!('^', env, |lhs, rhs| {
	match lhs.run(env)? {
		Value::Integer(base) => match (base, rhs.run(env)?.to_integer()?) {
			(_, Integer::MIN..=-1) => return Err(Error::DomainError("negative exponent")),
			(_, 0) => 1.into(),
			(0 | 1, _) => base.into(),

			#[cfg(feature = "checked-overflow")]
			(_, exponent) => {
				let exp =
					exponent.try_conv::<u32>().or(Err(Error::DomainError("negative exponent")))?;
				base.checked_pow(exp).ok_or(Error::IntegerOverflow)?.conv::<Value>()
			}
			#[cfg(not(feature = "checked-overflow"))]
			(_, exponent) => base.wrapping_pow(exponent as u32).conv::<Value>(),
		},
		#[cfg(feature = "arrays")]
		Value::List(llist) => {
			let max = rhs.run(env)?.to_integer()?;
			assert!(max >= 0, "todo, negative amounts");
			(0..max).map(Value::from).collect::<List>().into()
		}

		other => return Err(Error::TypeError(other.typename())),
	}
});

fn compare(lhs: &Value, rhs: &Value) -> Result<std::cmp::Ordering> {
	match lhs {
		Value::Integer(lnum) => Ok(lnum.cmp(&rhs.to_integer()?)),
		Value::Boolean(lbool) => Ok(lbool.cmp(&rhs.to_bool()?)),
		Value::SharedText(ltext) => Ok(ltext.cmp(&rhs.to_text()?)),
		#[cfg(feature = "arrays")]
		Value::List(llist) => {
			let rlist = rhs.to_array()?;
			for (lele, rele) in llist.iter().zip(rlist.iter()) {
				match compare(lele, rele)? {
					std::cmp::Ordering::Equal => {}
					other => return Ok(other),
				}
			}

			Ok(llist.len().cmp(&rlist.len()))
		}
		other => Err(Error::TypeError(other.typename())),
	}
}

/// **4.3.7** `<`  
pub const LESS_THAN: Function = function!('<', env, |lhs, rhs| {
	compare(&lhs.run(env)?, &rhs.run(env)?)? == std::cmp::Ordering::Less
});

/// **4.3.8** `>`  
pub const GREATER_THAN: Function = function!('>', env, |lhs, rhs| {
	compare(&lhs.run(env)?, &rhs.run(env)?)? == std::cmp::Ordering::Greater
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

		#[cfg(feature = "assign-to-prompt")]
		Value::Ast(ast) if ast.function().name == 'P' => env.add_to_prompt(value.to_text()?),

		#[cfg(feature = "arrays")]
		Value::Ast(ast) => return assign(&variable.run(env)?, value, env),
		#[cfg(feature = "arrays")]
		Value::List(list) => {
			if list.is_empty() {
				panic!("todo: error for this case");
			}
			let rhs = value.run(env)?.to_array()?;

			for (name, val) in list.as_slice().iter().zip(rhs.iter().cloned()) {
				assign(name, val, env)?;
			}

			match list.len().cmp(&rhs.len()) {
				std::cmp::Ordering::Equal => {}
				std::cmp::Ordering::Less => assign(
					list.as_slice().iter().last().unwrap(),
					rhs.as_slice()[list.len() - 1..].iter().cloned().collect::<List>().into(),
					env,
				)?,
				std::cmp::Ordering::Greater => {
					for extra in &list.as_slice()[rhs.len()..] {
						assign(extra, Value::default(), env)?;
					}
				}
			}
		}

		#[cfg(feature = "assign-to-anything")]
		_ => {
			let name = variable.run(env)?.to_text()?;
			env.lookup(&name)?.assign(value);
		}

		#[cfg(not(feature = "assign-to-anything"))]
		other => return Err(Error::TypeError(other.name())),
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
	let mut start = start.run(env)?.to_integer()?;
	let length = length
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative length")))?;

	#[cfg(feature = "arrays")]
	if let Value::List(list) = source {
		return Ok(if length == 0 {
			list.as_slice().get(start as usize).unwrap().clone()
		} else {
			list
				.as_slice()
				.get((start as usize)..(start as usize) + length)
				.expect("Todo: error")
				.iter()
				.cloned()
				.collect::<List>()
				.into()
		});
	}

	let string = source.to_text()?;

	if start < 0 && cfg!(feature = "negative-indexing") {
		start += string.len() as Integer;
	}
	let start = start.try_conv::<usize>().or(Err(Error::DomainError("negative start position")))?;

	match string.get(start..start + length) {
		Some(substring) => substring.to_owned(),

		#[cfg(feature = "no-oob-errors")]
		None => return Err(Error::IndexOutOfBounds { len: string.len(), index: start + length }),

		#[cfg(not(feature = "no-oob-errors"))]
		None => SharedText::default(),
	}
});

/// **4.5.1** `SET`  
pub const SET: Function = function!('S', env, |string, start, length, replacement| {
	let source = string.run(env)?;
	let mut start = start.run(env)?.to_integer()?;
	let length = length
		.run(env)?
		.to_integer()?
		.try_conv::<usize>()
		.or(Err(Error::DomainError("negative length")))?;
	let replacement_source = replacement.run(env)?;

	#[cfg(feature = "arrays")]
	if let Value::List(list) = source {
		if length == 0 {
			let mut dup = list.iter().cloned().collect::<Vec<Value>>();
			dup[start as usize] = replacement_source;
			return Ok(List::from(dup).into());
		}

		let replacement = replacement_source.to_array()?;

		let mut ret = Vec::new();
		ret.extend(list.iter().cloned().take((start as usize)));
		ret.extend(replacement.iter().cloned());
		ret.extend(list.iter().cloned().skip((start as usize) + length));

		return Ok(List::from(ret).into());
	}

	let string = source.to_text()?;
	let replacement = replacement_source.to_text()?;

	if start < 0 && cfg!(feature = "negative-indexing") {
		start += string.len() as Integer;
	}
	let start = start.try_conv::<usize>().or(Err(Error::DomainError("negative start position")))?;

	// TODO: `no-oob-errors` here
	// lol, todo, optimize me
	let mut builder = SharedText::builder();
	builder.push(&string.get(..start).unwrap());
	builder.push(&replacement);
	builder.push(&string.get(start + length..).unwrap());
	builder.finish()
});

/// **6.1** `VALUE`
#[cfg(feature = "value-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "value-function")))]
pub const VALUE: Function = function!('V', env, |arg| {
	let name = arg.run(env)?.to_text()?;
	env.lookup(&name)?
});

/// **Compiler extension**: SRAND
#[cfg(feature = "srand-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "srand-function")))]
pub const SRAND: Function = function!('X', env, |arg| {
	let seed = arg.run(env)?.to_integer()?;
	env.srand(seed);
	Value::default()
});

/// **6.4** `HANDLE`
#[cfg(feature = "handle-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "handle-function")))]
pub const HANDLE: Function = function!('H', env, |block, iferr| {
	const ERR_VAR_NAME: &'static crate::Text = unsafe { crate::Text::new_unchecked("_") };

	match block.run(env) {
		Ok(value) => value,
		Err(err) => {
			// This is fallible, as the error string might have had something bad.
			let errmsg = err.to_string().try_conv::<SharedText>()?;

			// Assign it to the error variable
			env.lookup(ERR_VAR_NAME).unwrap().assign(errmsg.into());

			// Finally, execute the RHS.
			iferr.run(env)?
		}
	}
});

/// **6.3** `USE`
#[cfg(feature = "use-function")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "use-function")))]
pub const USE: Function = function!('U', env, |arg| {
	let filename = arg.run(env)?.to_text()?;
	let contents = env.read_file(&filename)?;

	env.play(&contents)?
});
