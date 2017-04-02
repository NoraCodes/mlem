use types::*;

#[derive(PartialEq, Debug, Copy, Clone)]
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

#[derive(PartialEq, Debug, Copy, Clone)]
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

#[derive(PartialEq, Debug, Copy, Clone)]
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
    /// Gracefully shut down the machine
    Halt,
    /// An illegal instruction. Executing this is a Fault.
    Illegal,
}
