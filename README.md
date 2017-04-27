# MLeM

[![Crates.io version badge](https://img.shields.io/crates/v/mlem.svg)](https://crates.io/crates/mlem)
[![Docs.rs version badge](https://docs.rs/mlem/badge.svg)](https://docs.rs/mlem/)

The Machine Learning Machine is a 64-bit virtual Harvard-arch
machine for evolutionary algorithms to program against.

The machine has eight GPRs (`R0` through `R7`), a hardware stack with `SP` and `BP`, 
and hardware I/O with Input and Output. 
 
These I/O instructions write out whole `u64`s in big endian using `byteorder`.

## Example

This example shows a simple program being executed by the MLeM managed execution routine.

```
use mlem::{execute, Instruction, Address, Register, Outcome};
let input = vec![2, 2, 2, 2];
let expected = vec![4, 0];
let program = vec![
    // Get all input values
    Instruction::Input(Address::RegAbs(Register::R0)),
    Instruction::Input(Address::RegAbs(Register::R1)),
    Instruction::Input(Address::RegAbs(Register::R2)),
    Instruction::Input(Address::RegAbs(Register::R3)),
    // Perform arithmetic
    Instruction::Add(Address::RegAbs(Register::R0), Address::RegAbs(Register::R1)),
    Instruction::Sub(Address::RegAbs(Register::R2), Address::RegAbs(Register::R3)),
    // Output computed values
    Instruction::Output(Address::RegAbs(Register::R0)),
    Instruction::Output(Address::RegAbs(Register::R2)),
    // Halt
    Instruction::Halt
];
//!
let (outcome, _, output) = execute(program, input);
assert!(outcome == Outcome::Halt, "Program did not successfully halt! {:?}", outcome);
assert!(output == expected, "Program did not produce {:?} as expected, but rather {:?}.", expected, output);
```