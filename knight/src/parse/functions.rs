use crate::Value;
use super::{Parser, ParseResult};
use std::fmt::{self, Debug, Formatter};

cfg_if! {
	if #[cfg(feature="disallow-unicode")] {
		pub type Prefix = u8;
		type ParseMap = [Option<ParseFn>; Prefix::MAX as usize];
	} else {
		use std::collections::HashMap;

		pub type Prefix = char;
		type ParseMap = HashMap<Prefix, ParseFn>;
	}
}

pub type ParseFn = for<'env> fn(Prefix, &mut Parser<'env, '_>) -> ParseResult<Value<'env>>;

pub struct Functions(ParseMap);

impl Debug for Functions {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		struct PointerDebug(usize);
		impl Debug for PointerDebug {
			fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
				write!(f, "{:p}", self.0 as *const ())
			}
		}

		let mut map = f.debug_map();

		if_feature!("disallow-unicode" {
			for (chr, parsefn) in self.0.iter().enumerate() {
				if let &Some(parsefn) = parsefn {
					map.entry(&(chr as u8 as char), &PointerDebug(parsefn as usize));
				}
			}
		} else {
			for (chr, &parsefn) in self.0.iter() {
				map.entry(&chr, &PointerDebug(parsefn as usize));
			}
		});

		map.finish()
	}
}

impl Default for Functions {
	fn default() -> Self {
		// todo: actual default
		Self(Default::default())
	}
}

impl Functions {
	pub fn empty() -> Self {
		Self(Default::default())
	}

	pub fn register_parser(&mut self, prefix: Prefix, parsefn: ParseFn) {
		if_feature!("disallow-unicode" {
			self.0[prefix] = Some(parsefn);
		} else {
			self.0.insert(prefix, parsefn);
		});
	}

	// pub fn register_function(&mut self, prefix: Prefix, function: Function) {
	// 	self.register_parser(|)
	// 	if_feature!("disallow-unicode" {
	// 		self.0[prefix] = Some(parsefn);
	// 	} else {
	// 		self.0.insert(prefix, parsefn);
	// 	});
	// }

	pub fn get(&self, prefix: Prefix) -> Option<ParseFn> {
		if_feature!("disallow-unicode" {
			self.0[prefix]
		} else {
			self.0.get(&prefix).cloned()
		})
	}
}
