use std::io::{self, Result};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Source {
	path: Option<PathBuf>,
	contents: String,
}

impl Source {
	pub fn from_file(path: &Path) -> io::Result<Self> {
		let contents = std::fs::read_to_string(path)?;

		Ok(Self { path: Some(path.to_path_buf()), contents })
	}

	pub fn path(&self) -> Option<&Path> {
		self.path.as_deref()
	}

	pub fn contents(&self) -> &str {
		&self.contents
	}
}
