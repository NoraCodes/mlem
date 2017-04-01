//! # MLeM
//! The Machine Learning Machine is a 64-bit virtual Harvard-arch
//! machine for evolutionary algorithms to program against.
//!
//! The machine has eight GPRs (R0 through R7), a hardware stack with SP and BP, 
//! and hardware I/O with INPUT and OUTPUT. 

mod types;
pub use types::*;

mod machine;
#[cfg(test)]
mod test_machine;
pub use machine::*;

mod instructions;
pub use instructions::*;

#[cfg(test)]
mod tests {
}
