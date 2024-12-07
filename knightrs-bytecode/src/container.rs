cfg_if! {
if #[cfg(feature = "multithreaded")] {
	pub trait MaybeSendSync: Send + Sync {}
	impl<T: Send + Sync> MaybeSendSync for T {}
	pub type RefCount<T> = std::sync::Arc<T>;

} else {
	pub trait MaybeSendSync {}
	impl<T> MaybeSendSync for T {}
	pub type RefCount<T> = std::rc::Rc<T>;
}}
