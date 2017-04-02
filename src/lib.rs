//! # MLeM
//! The Machine Learning Machine is a 64-bit virtual Harvard-arch
//! machine for evolutionary algorithms to program against.
//!
//! The machine has eight GPRs (`R0` through `R7`), a hardware stack with SP and BP, 
//! and hardware I/O with Input and Output. 
//! 
//! These I/O instructions write out whole `u64`s in big endian using `byteorder`.

extern crate byteorder;

mod types;
pub use types::*;

mod machine;
#[cfg(test)]
mod test_machine;
pub use machine::*;

mod instructions;
pub use instructions::*;