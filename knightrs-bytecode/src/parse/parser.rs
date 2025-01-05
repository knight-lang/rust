// pub struct Parser<'env, 'src, 'path, 'gc> {
// 	env: &'env mut Environment<'gc>,
// 	filename: ProgramSource<'path>,
// 	source: &'src str, // can't use `KnStr` b/c it has a length limit.
// 	compiler: Compiler<'src, 'path, 'gc>,
// 	lineno: usize,

// 	// Start is loop begin, vec is those to jump to loop end
// 	loops: Vec<(JumpIndex, Vec<DeferredJump>)>,
// }
