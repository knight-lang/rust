use crate::{Value, value::UNDEFINED};

pub struct Variable {
	pub value: Value,
	pub name: *const u8,
	pub namelen: usize
}

impl Variable {
	pub unsafe fn name_str(&self) -> &str {
		use std::{str, slice};

		str::from_utf8_unchecked(slice::from_raw_parts(self.name, self.namelen))
	}
}

const NBUCKETS: usize = 65536;
const CAPACITY: usize = 256;

#[derive(Clone, Copy)]
struct Bucket {
	len: usize,
	vars: *mut [Variable; CAPACITY]
}

static mut MAP: [Bucket; NBUCKETS] = [Bucket { len: 0, vars: std::ptr::null_mut() }; NBUCKETS];

#[cfg(debug_assertions)]
static mut ENV_HAS_STARTED: bool = false;

pub unsafe fn startup() {
	// make sure we haven't started, and then set started to true.
	#[cfg(debug_assertions)] {
		debug_assert!(!ENV_HAS_STARTED);
		ENV_HAS_STARTED = true;
	}

	debug_assert_ne!(CAPACITY, 0);

	for i in 0..NBUCKETS {
		debug_assert_eq!(MAP[i].len, 0);

		MAP[i].vars = crate::malloc::<[Variable; CAPACITY]>() as *mut [Variable; CAPACITY];
	}
}

pub unsafe fn shutdown() {
	// make sure we've started, and then indicate we've shut down.
	#[cfg(debug_assertions)] {
		debug_assert!(ENV_HAS_STARTED);
		ENV_HAS_STARTED = false;
	}

	for i in 0..NBUCKETS {
		let bucket = &mut MAP[i];

		for i in 0..bucket.len {
			let var = &mut (*bucket.vars)[i];

			crate::shared::freestr(var.name as *mut u8, var.namelen);

			if var.value != UNDEFINED {
				crate::value::free(var.value);
			}
		}

		crate::free(bucket.vars);
		bucket.len = 0;
	}
}

pub unsafe fn fetch(name: *const u8, len: usize) -> *mut Variable {
	debug_assert_ne!(len, 0);

	let mut bucket = MAP[(crate::shared::hash(name, len) as usize) & (NBUCKETS - 1)];

	for i in 0..bucket.len {
		let var = (bucket.vars as *mut Variable).offset(i as isize);

		if crate::shared::strncmp((*var).name, name, len) == 0 {
			return var;
		}
	}

	if bucket.len == CAPACITY {
		panic!("too many variables encountered");
	}

	let var = (bucket.vars as *mut Variable).offset(bucket.len as isize);
	bucket.len += 1;

	(*var).name = crate::shared::strndup(name, len);
	(*var).value = UNDEFINED;

	var
}

pub unsafe fn assign(var: *mut Variable, value: Value) {
	if (*var).value != UNDEFINED {
		crate::value::free((*var).value);
	}

	(*var).value = value;
}

pub unsafe fn run(var: *mut Variable) -> Value {
	#[cfg(not(feature="reckless"))]
	assert_ne!((*var).value, UNDEFINED, "undefined variable '{}' ran", (*var).name_str());

	crate::value::clone((*var).value)
}
