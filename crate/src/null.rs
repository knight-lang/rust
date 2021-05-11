use crate::text::{Text, TextRef};

// notably not `ParitalOrd`/`ORd`, as Knight says null isnt comparable
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Null;
