#![allow(unused)]
use crate::parser::{ParseError, SourceLocation, VariableName};
use crate::program::{Compilable, Compiler, JumpWhen};
use crate::vm::Opcode;
use crate::Options;
use crate::Value;

pub struct Ast<'s, 'p, 'gc> {
	inner: AstInner<'s, 'p, 'gc>,
	location: SourceLocation<'p>,
}

pub enum AstInner<'s, 'p, 'gc> {
	Literal(Value<'gc>),
	Variable(VariableName<'s>),
	Then(Vec<Ast<'s, 'p, 'gc>>),
	Block(Box<Ast<'s, 'p, 'gc>>),

	SimpleAssign(VariableName<'s>, Box<Ast<'s, 'p, 'gc>>),
	And(Box<Ast<'s, 'p, 'gc>>, Box<Ast<'s, 'p, 'gc>>),
	Or(Box<Ast<'s, 'p, 'gc>>, Box<Ast<'s, 'p, 'gc>>),
	If(Box<Ast<'s, 'p, 'gc>>, Box<Ast<'s, 'p, 'gc>>, Box<Ast<'s, 'p, 'gc>>),
	While(Box<Ast<'s, 'p, 'gc>>, Box<Ast<'s, 'p, 'gc>>),

	SimpleOpcode(Opcode, Vec<Box<Ast<'s, 'p, 'gc>>>),
	Custom(Box<dyn Compilable<'s, 'p, 'gc>>),
}

unsafe impl<'s, 'p, 'gc> Compilable<'s, 'p, 'gc> for Ast<'s, 'p, 'gc> {
	fn compile(
		self,
		compiler: &mut Compiler<'s, 'p, 'gc>,
		opts: &Options,
	) -> Result<(), ParseError<'p>> {
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
				compiler.push_constant(crate::Value::NULL);
				Ok(())
			}
			AstInner::Custom(custom) => todo!(), //custom.compile(compiler, opts),
		}
	}
}
