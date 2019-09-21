//! # MLeM
//! The Machine Learning Machine is a 64-bit virtual Harvard-arch
//! machine for evolutionary algorithms to program against.
//!
//! The machine has eight GPRs (`R0` through `R7`), a hardware stack with SP and BP, 
//! and hardware I/O with Input and Output. 
//! 
//! These I/O instructions write out whole `u64`s in big endian.
//! 
//! # Example
//! ```
//! use mlem::{execute, Instruction, Address, Register, Outcome};
//! use mlem::Instruction::*;
//! use mlem::Address::*;
//! use mlem::Register::*;
//! let input = vec![2, 2, 2, 2];
//! let expected = vec![4, 0];
//! let program = vec![
//!     // Get all input values
//!     Input(RegAbs(R0)),
//!     Input(RegAbs(R1)),
//!     Input(RegAbs(R2)),
//!     Input(RegAbs(R3)),
//!     // Perform arithmetic
//!     Add(RegAbs(R0), RegAbs(R1)),
//!     Sub(RegAbs(R2), RegAbs(R3)),
//!     // Output computed values
//!     Output(RegAbs(R0)),
//!     Output(RegAbs(R2)),
//!     // Halt
//!     Halt
//! ];
//!
//! // The last value here is the maximum number of instructions to let the program run for.
//! let (outcome, cycles, output) = execute(program, input, Some(10));
//! assert!(outcome == Outcome::Halt, "Program did not successfully halt! {:?}", outcome);
//! assert!(output == expected, "Program did not produce {:?} as expected, but rather {:?}, in {} cycles.", expected, output, cycles);
//! ```

extern crate byteorder;
#[macro_use]
extern crate serde_derive;
extern crate serde_cbor;

mod virtual_machine;
//mod assembler;

pub use virtual_machine::{Outcome, execute};

#[cfg(test)]
mod test_instructions;

/// Represents a machine word - an atomic int, a pointer, etc.
/// Words are u64s; signed math has to do conversion.
pub type Word = u64;

/// A JumpLocation is a place on the instruction tape, which is a vector,
/// so it should be indexable.
pub type JumpLocation = usize;

/// Represents a program; a list of instructions, to be executed in order.
pub type Program = Vec<Instruction>;

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
/// Represents a place a value can come from: a register, a memory address, a pointer to memory stored in a register, or a literal value.
pub enum Address {
    /// A literal register, like R1.
    RegAbs(Register),
    /// A literal memory address, like 0x10.
    MemAbs(Word),
    /// A memory address stored in a register. This serves as one level of indirection; 
    /// for multiple indirection, multiple instructions must be used.
    MemReg(Register),
    /// A literal value. Writing to a literal value is a fault.
    Literal(Word),

}

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
/// Specifies a register in the machine. 
///
/// This doesn't include the instruction pointer. You have to use  use jump instructions
/// to mess with that.
pub enum Register {
    /// General purpouse register 0
    R0,
    /// General purpouse register 1
    R1,
    /// General purpouse register 2
    R2,
    /// General purpouse register 3
    R3,
    /// General purpouse register 4
    R4,
    /// General purpouse register 5
    R5,
    /// General purpouse register 6
    R6,
    /// General purpouse register 7
    R7,
    /// Stack position pointer
    SP,
    /// Stack base pointer
    BP,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
/// Possible instructions for the machine to execute.
/// For each instruction, the first operand is a, second is b, et cetera
pub enum Instruction {
    /// Increment IP.
    NoOp,
    /// Set a equal to 0
    Zero(Address),
    /// Set b equal to a
    Move(Address, Address),
    /// Push a onto the output
    Output(Address),
    /// Pop from the input into a
    Input(Address),
    /// Add the unsigned a to b, storing the result in a
    Add(Address, Address),
    /// Subtract the unsigned b from a, storing the result in a
    Sub(Address, Address),
    /// Uncontitionally jump to the position given by a
    Jump(Address),
    /// Jump to a if the value at b is 0
    JumpIfZero(Address, Address),
    /// Jump to a if the value at b is NOT zero
    JumpNotZero(Address, Address),
    /// Push a to the stack
    Push(Address),
    /// Pop a value from the stack into the given address
    Pop(Address),
    /// Gracefully shut down the machine
    Halt,
    /// An illegal instruction. Executing this is a Fault.
    Illegal,
}

