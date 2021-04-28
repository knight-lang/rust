use super::*;

pub struct TextBuilder {
	inner: *mut TextInner,
	len: usize,
}

impl Drop for TextBuilder {
	// in case the user doesn't actually build anything...
	fn drop(&mut self) {
		unsafe {
			if !self.inner().flags.contains(Flags::EMBEDDED) {
				drop(Box::from_raw(self.inner().kind.heap.ptr as *mut u8))
			}

			drop(Box::from_raw(self.inner));
		}
	}
}

impl TextBuilder {
	fn inner(&self) -> &TextInner {
		unsafe {
			&*self.inner
		}
	}

	pub fn capacity(&self) -> usize {
		let text = unsafe { Text(&*self.inner) };
		let capacity = text.len();

		std::mem::forget(text); // so we don't run the destructor

		capacity
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let builder = 
			if capacity <= EMBEDDED_LEN {
				Self::allocate_embedded(capacity)
			} else {
				Self::allocate_heap(capacity)
			};

		debug_assert_eq!(builder.inner().refcount.load(SeqCst), 1);
		debug_assert!(builder.inner().flags.contains(Flags::STRUCT_ALLOC));

		builder
	}

	fn allocate_embedded(capacity: usize) -> Self {
		debug_assert!(capacity <= (u8::MAX as usize));

		let inner =
			Box::into_raw(Box::new(TextInner {
				refcount: AtomicUsize::new(1),
				flags: Flags::STRUCT_ALLOC | Flags::EMBEDDED,
				kind: TextKind {
					embed: TextKindEmbedded {
						len: capacity as u8,
						data: [0u8; EMBEDDED_LEN]
					}
				}
			}));

		Self { inner, len: 0 }
	}

	fn allocate_heap(capacity: usize) -> Self {
		let inner = 
			Box::into_raw(Box::new(TextInner {
				refcount: AtomicUsize::new(1),
				flags: Flags::STRUCT_ALLOC,
				kind: TextKind {
					heap: TextKindPointer {
						_padding: [0u8; TEXT_KIND_POINTER_PADDING_LEN],
						len: capacity,
						ptr: Box::into_raw(Vec::<u8>::with_capacity(capacity).into_boxed_slice()) as *mut u8
					}
				}
			}));

		Self { inner, len: 0 }
	}

	pub fn concat(&mut self, mut data: &str) -> Result<usize, InvalidByte> {
		validate(data.as_bytes())?;

		let cap = self.capacity();

		if self.len == cap {
			return Ok(0);
		} 

		if cap < data.len() + self.len {
			data = &data[..cap - self.len];
		}

		unsafe {
			debug_assert_eq!(self.len as isize as usize, self.len);
			std::ptr::copy_nonoverlapping(data.as_ptr(), self.as_ptr_mut().offset(self.len as isize), data.len());
		}

		self.len += data.len();
		Ok(data.len())
	}

	pub fn as_ptr_mut(&mut self) -> *mut u8 {
		let text = unsafe { Text(&*self.inner) };
		let ptr = text.as_ptr();

		std::mem::forget(text); // so we don't run the destructor

		ptr as *mut u8
	}

	#[inline(always)]
	pub fn build(self) -> Text {
		let inner_ptr = self.inner;

		std::mem::forget(self); // so we don't run the destructor

		unsafe {
			Text(&*(inner_ptr as *const TextInner))
		}
	}
}


// #[repr(C)]
// struct TextInner {
// 	refcount: AtomicUsize,
// 	flags: Flags,
// 	kind: TextKind
// }

// #[repr(C)]
// union TextKind {
// 	embed: TextKindEmbedded,
// 	ptr:   TextKindPointer
// }

// const EMBEDDED_LEN: usize = 64 - size_of::<AtomicUsize>() - size_of::<Flags>() - size_of::<u8>();

// sa::const_assert!(EMBEDDED_LEN <= u8::MAX as usize);

// #[derive(Clone, Copy)]
// #[repr(C, packed)]
// struct TextKindEmbedded {
// 	len: u8,
// 	data: [u8; EMBEDDED_LEN]
// }


// const TEXT_KIND_POINTER_PADDING_LEN: usize = align_of::<usize>() - align_of::<Flags>();

// sa::const_assert_eq!(
// 	(align_of::<AtomicUsize>() + align_of::<Flags>() + TEXT_KIND_POINTER_PADDING_LEN) % align_of::<usize>(),
// 	0
// );

// #[derive(Clone, Copy)]
// #[repr(C, packed)]
// struct TextKindPointer {
// 	_padding: [u8; TEXT_KIND_POINTER_PADDING_LEN],
// 	len: usize,
// 	ptr: *const u8
// }
