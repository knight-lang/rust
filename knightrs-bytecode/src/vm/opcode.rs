/// Opcodes represent different instructions that the [`Vm`](crate::vm::Vm) understands.
// Implementation note: They're intentionally constructed in a special way, so as to make accessing
// information like their arity super easy. More precisely, they're structured like:
//
//   opcode := `AAIIIIIO`
//
// where `A` is the arity, `I` is index, and `O` is if it takes an offset. Note that functions which
// take more than 3 arguments need to pop their arguments off manually.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
#[rustfmt::skip]
pub enum Opcode {
	// Builtins
	PushConstant = opcode(0, 0, true),
	Jump         = opcode(1, 0, true),
	JumpIfTrue   = opcode(2, 1, true),
	JumpIfFalse  = opcode(3, 1, true),
	GetVar       = opcode(4, 0, true),
	SetVar       = opcode(5, 0, true),    // no opcode cause top of stack
	SetVarPop    = opcode(6, 1, true), // same as setvar but it pips
	#[cfg(feature = "extensions")]
	AssignDynamic = opcode(7, 0, true), // offset is the type to use

	// Arity 0
	Prompt = opcode(1, 0, false),
	Random = opcode(2, 0, false),
	Dup = opcode(3, 0, false),  // doesnt have an arity cause that pops
	Dump = opcode(5, 0, false), // special-cased in `function.rs` so it doesn't pop.

	// Arity 1
	#[cfg(feature = "stacktrace")]
	Return = opcode(0, 1, false),
	#[cfg(not(feature = "stacktrace"))]
	Return = opcode(6, 0, false),

	Call   = opcode(1, 1, false),
	Quit   = opcode(2, 1, false),
	Output = opcode(3, 1, false),
	Length = opcode(4, 1, false),
	Not    = opcode(5, 1, false),
	Negate = opcode(6, 1, false),
	Ascii  = opcode(7, 1, false),
	Box    = opcode(8, 1, false),
	Head   = opcode(9, 1, false),
	Tail   = opcode(10, 1, false),
	Pop    = opcode(11, 1, false),

	#[cfg(feature = "extensions")]
	Eval   = opcode(12, 1, false),
	#[cfg(feature = "extensions")]
	Value  = opcode(13, 1, false),

	// Arity 2
	Add           = opcode(0, 2, false),
	Sub           = opcode(1, 2, false),
	Mul           = opcode(2, 2, false),
	Div           = opcode(3, 2, false),
	Mod           = opcode(4, 2, false),
	Pow           = opcode(5, 2, false),
	Lth           = opcode(6, 2, false),
	Gth           = opcode(7, 2, false),
	Eql           = opcode(8, 2, false),
	#[cfg(feature = "extensions")]
	SetDynamicVar = opcode(9, 2, false),

	// Arity 3
	Get = opcode(0, 3, false),

	// Arity 4
	Set = opcode(0, 4, false),
}

#[cfg(feature = "extensions")]
#[non_exhaustive]
#[repr(u8)]
pub enum DynamicAssignment {
	Output,
	Prompt,
	Random,
	System,
}

// If it goes higher than this, we need to rework the structure of the opcode.
const fn opcode(id: u8, arity: u8, takes_offset: bool) -> u8 {
	assert!(arity as usize <= 0b111, "7 is max arity that can be taken");
	assert!(id <= 0b11111, "too many IDs of a given arity will clobber stuff");

	(arity << 5) | (id << 1) | (takes_offset as u8)
}

impl Opcode {
	/// The amount of arguments the opcode expects the stack to have.
	#[inline]
	pub const fn arity(self) -> usize {
		((self as u8) >> 5) as usize
	}

	/// Whether the opcode takes an offset
	#[inline]
	pub const fn takes_offset(self) -> bool {
		(self as u8) & 1 != 0
	}

	/// Returns the [`Opcode`] from the byte, without checking to see if it's a valid [`Opcode`].
	///
	/// # Safety
	/// The caller must ensure that `byte` corresponds to a valid [`Opcode`] representation.
	#[cfg_attr(not(debug_assertions), inline)]
	pub unsafe fn from_byte_unchecked(byte: u8) -> Self {
		debug_assert!(
			// Builtins
			byte == Self::PushConstant as u8
				|| byte == Self::Jump as u8
				|| byte == Self::JumpIfTrue as u8
				|| byte == Self::JumpIfFalse as u8
				|| byte == Self::GetVar as u8
				|| byte == Self::SetVar as u8
				|| byte == Self::SetVarPop as u8

			// Arity 0
				|| byte == Self::Prompt as u8
				|| byte == Self::Random as u8
				|| byte == Self::Dup as u8
				|| byte == Self::Return as u8

			// Arity 1
				|| byte == Self::Call as u8
				|| byte == Self::Quit as u8
				|| byte == Self::Dump as u8
				|| byte == Self::Output as u8
				|| byte == Self::Length as u8
				|| byte == Self::Not as u8
				|| byte == Self::Negate as u8
				|| byte == Self::Ascii as u8
				|| byte == Self::Box as u8
				|| byte == Self::Head as u8
				|| byte == Self::Tail as u8
				|| byte == Self::Pop as u8
				|| { #[cfg(feature = "extensions")] {
					   byte == Self::Eval as u8
					|| byte == Self::Value as u8
					|| byte == Self::SetDynamicVar as u8
					|| byte == Self::AssignDynamic as u8
				}
				#[cfg(not(feature = "extensions"))] { false } }

			// Arity 2
				|| byte == Self::Add as u8
				|| byte == Self::Sub as u8
				|| byte == Self::Mul as u8
				|| byte == Self::Div as u8
				|| byte == Self::Mod as u8
				|| byte == Self::Pow as u8
				|| byte == Self::Lth as u8
				|| byte == Self::Gth as u8
				|| byte == Self::Eql as u8

			// Arity 3
				|| byte == Self::Get as u8

			// Arity 4
				|| byte == Self::Set as u8
		);

		// SAFETY: `Opcode` is `#[repr(u8)]`, and the caller ensures that `byte` is actually a valid
		// opcode, so this transmutation is safe.
		return unsafe { std::mem::transmute::<u8, Opcode>(byte) };
	}
}
