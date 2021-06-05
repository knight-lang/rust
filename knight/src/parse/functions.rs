use super::{TokenizeFn, Character};
use std::fmt::{self, Debug, Formatter};

cfg_if! {
	if #[cfg(feature="disallow-unicode")] {
		type ParseMap = [Option<TokenizeFn>; Character::MAX as usize + 1];
	} else {
		use std::collections::HashMap;
		type ParseMap = HashMap<Character, TokenizeFn>;
	}
}

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
		Self::empty()
	}
}

impl Functions {
	pub fn empty() -> Self {
		Self(
			#[cfg(feature="disallow-unicode")]
			{ [None; Character::MAX as usize + 1] },
			#[cfg(not(feature="disallow-unicode"))]
			{ Default::default() }
		)
	}

	pub fn register_parser(&mut self, prefix: Character, parsefn: TokenizeFn) {
		if_feature!("disallow-unicode" {
			self.0[prefix as usize] = Some(parsefn);
		} else {
			self.0.insert(prefix, parsefn);
		});
	}

	// pub fn register_function(&mut self, prefix: Character, function: Function) {
	// 	self.register_parser(|)
	// 	if_feature!("disallow-unicode" {
	// 		self.0[prefix] = Some(parsefn);
	// 	} else {
	// 		self.0.insert(prefix, parsefn);
	// 	});
	// }

	pub fn get(&self, prefix: Character) -> Option<TokenizeFn> {
		if_feature!("disallow-unicode" {
			self.0[prefix as usize]
		} else {
			self.0.get(&prefix).cloned()
		})
	}
}
