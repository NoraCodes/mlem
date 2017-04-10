//! Types to make definitons more expressive.

/// Represents a machine word - an atomic int, a pointer, etc.
/// Words are u64s; signed math has to do conversion.
pub type Word = u64;

/// A JumpLocation is a place on the instruction tape, which is a vector,
/// so it should be indexable.
pub type JumpLocation = usize;

/// Represents a program; a list of instructions, to be executed in order.
use instructions::Instruction;
pub type Program = Vec<Instruction>;

