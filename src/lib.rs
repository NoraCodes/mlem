//! # MLeM
//! The Machine Learning Machine is a 64-bit virtual Harvard-arch
//! machine for evolutionary algorithms to program against.
//!
//! The machine has eight GPRs (`R0` through `R7`), a hardware stack with SP and BP, 
//! and hardware I/O with Input and Output. 
//! 
//! These I/O instructions write out whole `u64`s in big endian using `byteorder`.
//! 
//! # Example
//! ```
//! use mlem::{execute, Instruction, Address, Register, Outcome};
//! let input = vec![2, 2, 2, 2];
//! let expected = vec![4, 0];
//! let program = vec![
//!     // Get all input values
//!     Instruction::Input(Address::RegAbs(Register::R0)),
//!     Instruction::Input(Address::RegAbs(Register::R1)),
//!     Instruction::Input(Address::RegAbs(Register::R2)),
//!     Instruction::Input(Address::RegAbs(Register::R3)),
//!     // Perform arithmetic
//!     Instruction::Add(Address::RegAbs(Register::R0), Address::RegAbs(Register::R1)),
//!     Instruction::Sub(Address::RegAbs(Register::R2), Address::RegAbs(Register::R3)),
//!     // Output computed values
//!     Instruction::Output(Address::RegAbs(Register::R0)),
//!     Instruction::Output(Address::RegAbs(Register::R2)),
//!     // Halt
//!     Instruction::Halt
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

mod types;
pub use types::*;

mod machine;
#[cfg(test)]
mod test_machine;
pub use machine::*;

mod instructions;
#[cfg(test)]
mod test_instructions;
pub use instructions::*;

/// Given a Program (that is, a Vec of Instructions), this function will manage creating a Machine and hooking up its 
/// Input and Output for you. It returns a tuple of the final outcome of the program, the number of instructions executed, and
/// a Vector of the output.
pub fn execute(program: Program, input: Vec<u64>, limit: Option<u64>) -> (Outcome, u64, Vec<u64>) {
    use std::io::{Cursor, Seek};
    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
    // Create and fill a buffer of u8s with the values of the given u64s, in big endian
    let mut internal_input = Cursor::new(Vec::with_capacity(input.len() * 8));
    for v in input {
        internal_input.write_u64::<BigEndian>(v).unwrap();
    }
    internal_input.seek(std::io::SeekFrom::Start(0)).unwrap();

    // Create output buffer
    let mut internal_output = Cursor::new(Vec::new());
    
    // Actually run the machine.
    let o;
    let cycles;
    {
        let mut m = Machine::new(128, &mut internal_input, &mut internal_output);

        m.load_program(program);
        let actual_limit = limit.unwrap_or(u64::max_value());
        let (a, b) = m.run_for(actual_limit);
        o = a;
        cycles = b;
    }
    // Compose output into u64 values
    let mut output = Vec::new();
    internal_output.seek(std::io::SeekFrom::Start(0)).unwrap();
    while let Ok(v) = internal_output.read_u64::<BigEndian>() {
        output.push(v);
    }

    (o, cycles, output)
}