use std::mem::MaybeUninit;
use std::alloc::{self, Layout};

extern "C" {
	pub fn strncmp(lhs: *const u8, rhs: *const u8, len: usize) -> i32;
	pub fn strcmp(lhs: *const u8, rhs: *const u8) -> i32;
	pub fn memcpy(dst: *mut u8, src: *const u8, amnt: usize) -> *mut u8;
}

pub unsafe fn hash_acc(input: *const u8, mut len: usize, mut hash: u64) -> u64 {
	// murmur hash

	while len != 0 {
		len -= 1;

		hash ^= *input as u64;
		hash = hash.wrapping_mul(0x5bd1e9955bd1e995);
		hash ^= hash >> 47;

	}	

	hash
}

pub unsafe fn hash(input: *const u8, len: usize) -> u64 {
	hash_acc(input, len, 525201411107845655)
}

pub unsafe fn malloc<T>() -> *mut MaybeUninit<T> {
	alloc::alloc(Layout::new::<T>()) as *mut _
}

pub unsafe fn free<T>(ptr: *mut T) {
	alloc::dealloc(ptr as *mut u8, Layout::new::<T>());
}

pub unsafe fn freestr(str: *mut u8, len: usize) {
	alloc::dealloc(str, alloc::Layout::from_size_align_unchecked(len, 1));
}

pub unsafe fn strndup(src: *const u8, len: usize) -> *mut u8 {
	debug_assert_ne!(len, 0);

	if len == 0 {
		return std::ptr::null_mut();
	}

	let dup = alloc::alloc(Layout::from_size_align_unchecked(len + 1, 1));
	memcpy(dup, src, len);
	*dup.offset(len as isize) = b'\0';

	dup
}

