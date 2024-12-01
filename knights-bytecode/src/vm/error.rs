pub struct RuntimeError {
	kind: RuntimeErrorKind,

	#[cfg(feature = "stacktrace")]
	stacktrace: Vec<SourceLocation>,
}

pub enum RuntimeErrorKind {}
