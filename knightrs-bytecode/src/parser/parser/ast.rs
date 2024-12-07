use crate::parser::{ParseError, SourceLocation, VariableName};
use crate::program::{Compilable, Compiler, JumpWhen};
use crate::vm::Opcode;
use crate::Options;
use crate::Value;

pub struct Ast<'s, 'p> {
	inner: AstInner<'s, 'p>,
	location: SourceLocation<'p>,
}

pub enum AstInner<'s, 'p> {
	Literal(Value),
	Variable(VariableName<'s>),
	Then(Vec<Ast<'s, 'p>>),
	Block(Box<Ast<'s, 'p>>),

	SimpleAssign(VariableName<'s>, Box<Ast<'s, 'p>>),
	And(Box<Ast<'s, 'p>>, Box<Ast<'s, 'p>>),
	Or(Box<Ast<'s, 'p>>, Box<Ast<'s, 'p>>),
	If(Box<Ast<'s, 'p>>, Box<Ast<'s, 'p>>, Box<Ast<'s, 'p>>),
	While(Box<Ast<'s, 'p>>, Box<Ast<'s, 'p>>),

	SimpleOpcode(Opcode, Vec<Box<Ast<'s, 'p>>>),
	Custom(Box<dyn Compilable<'s, 'p>>),
}

unsafe impl<'s, 'p> Compilable<'s, 'p> for Ast<'s, 'p> {
	fn compile(self, compiler: &mut Compiler<'s, 'p>, opts: &Options) -> Result<(), ParseError<'p>> {
		match self.inner {
			AstInner::Literal(value) => {
				compiler.push_constant(value);
				Ok(())
			}
			AstInner::Variable(var) => (var, self.location).compile(compiler, opts),
			AstInner::Then(stmts) => {
				let mut stmts = stmts.into_iter();
				// TODO: check for simple assign
				stmts.next().unwrap().compile(compiler, opts)?;
				for stmt in stmts {
					unsafe { compiler.opcode_without_offset(Opcode::Pop) }
					stmt.compile(compiler, opts)?;
				}
				Ok(())
			}
			AstInner::SimpleOpcode(op, args) => {
				for arg in args {
					arg.compile(compiler, opts)?;
				}
				// safety: todo
				unsafe {
					compiler.opcode_without_offset(op);
				}
				Ok(())
			}

			AstInner::Block(value) => {
				todo!();
			}

			AstInner::SimpleAssign(name, value) => {
				value.compile(compiler, opts)?;
				unsafe {
					compiler.set_variable(name, &opts);
				}
				Ok(())
			}
			AstInner::And(left, right) => {
				left.compile(compiler, opts)?;
				unsafe {
					compiler.opcode_without_offset(Opcode::Dup);
				}
				let end = compiler.defer_jump(JumpWhen::False);
				unsafe {
					// delete the value we dont want
					compiler.opcode_without_offset(Opcode::Pop);
				}
				right.compile(compiler, opts)?;
				unsafe {
					end.jump_to_current(compiler);
				}
				Ok(())
			}
			AstInner::Or(left, right) => {
				left.compile(compiler, opts)?;
				unsafe {
					compiler.opcode_without_offset(Opcode::Dup);
				}
				let end = compiler.defer_jump(JumpWhen::True);
				unsafe {
					// delete the value we dont want
					compiler.opcode_without_offset(Opcode::Pop);
				}
				right.compile(compiler, opts)?;
				unsafe {
					end.jump_to_current(compiler);
				}
				Ok(())
			}

			AstInner::If(cond, iftrue, iffalse) => {
				cond.compile(compiler, opts)?;
				let to_false = compiler.defer_jump(JumpWhen::False);
				iftrue.compile(compiler, opts)?;
				let to_end = compiler.defer_jump(JumpWhen::Always);
				unsafe {
					to_false.jump_to_current(compiler);
				}
				iffalse.compile(compiler, opts)?;
				unsafe {
					to_end.jump_to_current(compiler);
				}
				Ok(())
			}

			AstInner::While(cond, body) => {
				let while_start = compiler.jump_index();

				cond.compile(compiler, opts)?;
				let deferred = compiler.defer_jump(JumpWhen::False);
				compiler.loops.push((while_start, vec![deferred]));

				body.compile(compiler, opts)?;
				unsafe {
					compiler.opcode_without_offset(Opcode::Pop);
					compiler.jump_to(JumpWhen::Always, while_start);
				}

				// jump all `break`s to the end
				for deferred in compiler.loops.pop().unwrap().1 {
					unsafe {
						deferred.jump_to_current(compiler);
					}
				}
				compiler.push_constant(crate::Value::Null);
				Ok(())
			}
			AstInner::Custom(custom) => todo!(), //custom.compile(compiler, opts),
		}
	}
}
