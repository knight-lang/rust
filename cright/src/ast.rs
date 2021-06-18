use crate::func::{Function, MAX_ARGC};
use crate::Value;
use std::ptr;

pub struct Ast {
	pub refcount: u32,
	pub func: *const Function,
	pub args: [Value; MAX_ARGC]
}

const FREE_CACHE_LEN: usize = 32;

static mut FREED_ASTS: [[*mut Ast; FREE_CACHE_LEN]; MAX_ARGC + 1] = [[ptr::null_mut(); FREE_CACHE_LEN]; MAX_ARGC + 1];

pub unsafe fn cleanup() {
	for i in 0..MAX_ARGC {
		for j in 0..FREE_CACHE_LEN {
			let ast = FREED_ASTS[i][j];

			if !ast.is_null() {
				debug_assert_eq!((*ast).refcount, 0);
				crate::free(ast);
			}
		}
	}
}

pub unsafe fn alloc(func: *const Function) -> *mut Ast {
	let arity = (*func).arity;

	#[cfg(feature="cache-asts")]
	for i in 0..FREE_CACHE_LEN {
		let ast = FREED_ASTS[arity][i];

		if ast.is_null() {
			continue;
		}

		FREED_ASTS[arity][i] = ptr::null_mut();

		debug_assert_eq!((*ast).refcount, 0);
		(*ast).refcount += 1;

		return ast;
	}

	let ast = crate::malloc::<Ast>() as *mut Ast;

	(*ast).refcount = 1;
	(*ast).func = func;

	ast
}

pub unsafe fn clone(ast: *mut Ast) -> *mut Ast {
	(*ast).refcount += 1;

	ast
}

pub unsafe fn free(ast: *mut Ast) {
	let mut astm = &mut *ast;
	debug_assert_ne!(astm.refcount, 0);

	astm.refcount -= 1;

	if astm.refcount != 0 {
		return;
	}

	let arity = (*astm.func).arity;

	for i in 0..arity {
		crate::value::free(astm.args[i]);
	}

	#[cfg(feature="cache-asts")]
	for i in 0..FREE_CACHE_LEN {
		if FREED_ASTS[arity][i].is_null() {
			FREED_ASTS[arity][i] = ast;
			return;
		}
	}

	crate::free(ast);
}

pub unsafe fn run(ast: *mut Ast) -> Value {
	((*(*ast).func).func)(&(*ast).args as *const Value)
}
