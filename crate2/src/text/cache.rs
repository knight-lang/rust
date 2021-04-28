use super::*;

use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::sync::Mutex;
use std::borrow::Borrow;

static STRING_CACHE: OnceCell<Mutex<HashSet<Text>>> = OnceCell::new();

pub fn fetch_or_insert<T: Borrow<[u8]>>(data: T, insert: impl FnOnce(T) -> Box<[u8]>) -> Text {
	debug_assert!(validate(data.borrow()).is_ok());

	let mut cache = STRING_CACHE.get_or_init(Default::default).lock().unwrap();

	if let Some(text) = cache.get(data.borrow()) {
		return text.clone();
	}

	let data = Text::new_owned(insert(data)).unwrap();

	cache.insert(data.clone());

	data
}