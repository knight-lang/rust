use std::mem::size_of;
use std::alloc::{self, Layout};

pub const FL_STRUCT_ALLOC: u32 = 1;
pub const FL_EMBED: u32        = 2;
pub const FL_STATIC: u32       = 4;
pub const FL_CACHED: u32       = 8;

pub const PADDING_LEN: usize = 16;
pub const EMBEDDED_LEN: usize = size_of::<*const u8>() + PADDING_LEN - 1;


#[macro_export]
macro_rules! new_embed {
	($len:literal, $full:expr) => ($crate::Text {
		refcount: 0, // irrelevant
		flags: $crate::text::FL_EMBED,
		len: $len,
		data: $crate::text::TextData { embed: $full }
	});
}

#[repr(C, align(8))]
pub struct Text {
	pub refcount: u32,
	pub flags: u32,
	pub len: usize,
	pub data: TextData,
}

#[repr(C)]
pub union TextData {
	pub embed: [u8; EMBEDDED_LEN],
	pub alloc: *mut u8
}

impl Text {
	pub unsafe fn str(&self) -> &str {
		use std::{str, slice};

		str::from_utf8_unchecked(
			slice::from_raw_parts(ptr(self as *const _ as *mut _),
				self.len as usize
			))
	}
}

pub static mut EMPTY: Text = new_embed!(0, [0; EMBEDDED_LEN]);

const CACHE_MAXLEN: usize = 32;
const CACHE_LINELEN: usize = 1<<14;

static mut CACHE: [[*mut Text; CACHE_LINELEN]; CACHE_MAXLEN] = [[std::ptr::null_mut(); CACHE_LINELEN]; CACHE_MAXLEN];

unsafe fn cache_lookup(hash: u64, len: usize) -> *mut *mut Text {
	debug_assert_ne!(len, 0);
	debug_assert!(len <= CACHE_MAXLEN);

	&mut CACHE[len - 1][(hash as usize) & (CACHE_LINELEN - 1)] as *mut _
}

pub unsafe fn text_cache_lookup(hash: u64, len: usize) -> *mut Text {
	if len == 0 || CACHE_MAXLEN < len {
		std::ptr::null_mut()
	} else {
		*cache_lookup(hash, len)
	}
}

unsafe fn get_cache_slot(str: *const u8, len: usize) -> *mut *mut Text {
	return cache_lookup(crate::shared::hash(str, len), len);
}

pub unsafe fn ptr(text: *mut Text) -> *mut u8 {
	if (*text).flags & FL_EMBED != 0 {
		&mut (*text).data.embed as *mut _ as *mut u8
	} else {
		(*text).data.alloc
	}
}

pub unsafe fn equal(lhs: *const Text, rhs: *const Text) -> bool {
	if lhs == rhs {
		return true;
	}

	if (*lhs).len != (*rhs).len {
		return false;
	}

	std::slice::from_raw_parts(ptr(lhs as *mut _) as *const _, (*lhs).len)
		== std::slice::from_raw_parts(ptr(rhs as *mut _) as *const _, (*rhs).len)
}

unsafe fn allocate_heap_text(str: *mut u8, len: usize) -> *mut Text {
	debug_assert!(!str.is_null());
	debug_assert_ne!(len, 0);

	let mut text = crate::malloc::<Text>() as *mut Text;

	(*text).flags = FL_STRUCT_ALLOC;
	(*text).refcount = 1;
	(*text).len = len;
	(*text).data.alloc = str;

	text
}

unsafe fn allocate_embed_text(len: usize) -> *mut Text {
	debug_assert_ne!(len, 0);

	let mut text = crate::malloc::<Text>() as *mut Text;

	(*text).flags = FL_STRUCT_ALLOC | FL_EMBED;
	(*text).refcount = 1;
	(*text).len = len;

	text
}

unsafe fn deallocate_text(text: *mut Text) {
	debug_assert!(!text.is_null());
	debug_assert_eq!((*text).refcount, 0); // don't dealloc live strings...

	let flags = (*text).flags;

	// If the struct isn't actually allocated, then return.
	if flags & FL_STRUCT_ALLOC != 0 {
		// Sanity check, as these are the only two non-struct-alloc flags.
		debug_assert_ne!(flags & (FL_EMBED | FL_STATIC), 0);
		return;
	}

	// If we're not embedded, free the allocated string
	if flags & FL_EMBED == 0{
		crate::free((*text).data.alloc);
	}

	// Finally free the entire struct itself.
	crate::free(text);
}

unsafe fn evict_text(text: *mut Text) {
	// we only cache allocated strings.
	debug_assert_ne!((*text).flags & FL_STRUCT_ALLOC, 0);

	if (*text).refcount == 0 {
		deallocate_text(text)
	} else {
		debug_assert_ne!((*text).flags & FL_CACHED, 0);

		(*text).flags -= FL_CACHED;
	}
}

pub unsafe fn alloc(len: usize) -> *mut Text {
	if len == 0 {
		return &mut EMPTY as *mut Text
	}

	if len <= EMBEDDED_LEN {
		allocate_embed_text(len)
	} else {
		allocate_heap_text(alloc::alloc(Layout::from_size_align_unchecked(len + 1, 1)), len)
	}
}

pub unsafe fn cache(text: *mut Text) {
	debug_assert_ne!((*text).len, 0);

	if (*text).len < CACHE_MAXLEN {
		return;
	}

	let cacheline = get_cache_slot(ptr(text), (*text).len);

	if !(*cacheline).is_null() {
		evict_text(*cacheline)
	}

	(*text).flags |= FL_CACHED;
	*cacheline = text;
}

extern "C" {
	#[cfg(debug_assertions)]
	fn strlen(str: *const u8) -> usize;
	fn strcmp(lhs: *const u8, rhs: *const u8) -> i32;
}

pub unsafe fn new_owned(str: *mut u8, len: usize) -> *mut Text {
	debug_assert!(!str.is_null());
	debug_assert_eq!(strlen(str), len);

	if len == 0 {
		alloc::dealloc(str, Layout::from_size_align_unchecked(1, 1));
		return &mut EMPTY as *mut Text;
	}

	if CACHE_MAXLEN < len {
		return allocate_heap_text(str, len);
	}

	let cacheline = get_cache_slot(str, len);
	let text = *cacheline;

	if text.is_null() {
		if strcmp(ptr(text), str) == 0 {
			alloc::dealloc(str, Layout::from_size_align_unchecked(len + 1, 1));
			(*text).refcount += 1;
			return text;
		}

		evict_text(text);
	}

	let text = allocate_heap_text(str, len);
	*cacheline = text;
	(*text).flags |= FL_CACHED;

	text
}

pub unsafe fn new_borrowed(str: *const u8, len: usize) -> *mut Text {
	debug_assert!(!str.is_null());

	if len == 0 {
		return &mut EMPTY as *mut Text;
	}

	if CACHE_MAXLEN < len {
		return allocate_heap_text(crate::shared::strndup(str, len), len);
	}

	let cacheline = get_cache_slot(str, len);
	let text = *cacheline;

	if text.is_null() {
		if strcmp(ptr(text), str) == 0 {
			(*text).refcount += 1;
			return text;
		}

		evict_text(text);
	}	

	let text = alloc(len);
	*cacheline = text;
	(*text).flags |= FL_CACHED;

	std::ptr::copy_nonoverlapping(str, ptr(text), len);
	*ptr(text).offset(len as isize) = b'\0';

	text
}

pub unsafe fn free(text: *mut Text) {
	(*text).refcount -= 1;

	if (*text).refcount == 0 && (*text).flags & FL_CACHED != 0 {
		deallocate_text(text)
	}
}

pub unsafe fn clone(text: *mut Text) -> *mut Text {
	debug_assert_eq!((*text).flags & FL_STATIC, 0);

	(*text).refcount += 1;

	text
}

pub unsafe fn clone_static(text: *mut Text) -> *mut Text {
	if (*text).flags & FL_STATIC == 0 {
		text
	} else {
		new_borrowed(ptr(text), (*text).len)
	}
}

pub unsafe fn cleanup() {
	for i in 0..CACHE_MAXLEN {
		for j in 0..CACHE_LINELEN {
			let text = CACHE[i][j];

			if !text.is_null() {
				debug_assert_ne!((*text).flags & FL_STRUCT_ALLOC, 0);

				if (*text).refcount == 0 {
					deallocate_text(text);
				}
			}
		}
	}
}

pub unsafe fn is_equal(lhs: *const Text, rhs: *const Text) -> bool {
	if lhs == rhs {
		return true;
	}

	if (*lhs).len != (*rhs).len {
		return false;
	}

	extern "C" {
		fn memcmp(lhs: *const u8, rhs: *const u8, amnt: usize) -> i32;
	}

	memcmp(ptr(lhs as *mut _) as *const _, ptr(rhs as *mut _) as *const _, (*lhs).len) == 0
}


