pub struct Ast<'env> {
	x: &'env ()
}

impl<'env> Ast<'env> {
	pub fn into_raw(self) -> *const () {
		todo!()
	}

	pub unsafe fn from_raw(raw: *const ()) -> Self {
		todo!()
	}
}