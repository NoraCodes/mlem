use crate::{Address, Instruction, Program, Register};
use serde_cbor::de::from_slice;
use serde_cbor::ser::to_vec_sd;

#[test]
fn test_cbor_serde() {
    let program: Program = vec![
        Instruction::NoOp,
        Instruction::Input(Address::MemAbs(1)), // Input to position 1
        Instruction::Input(Address::RegAbs(Register::R1)), // Input to position *R1
        Instruction::Add(Address::RegAbs(Register::R1), Address::MemAbs(1)), // Add
        Instruction::Output(Address::RegAbs(Register::R1)),
        Instruction::Halt,
    ];

    let serialized = to_vec_sd(&program).unwrap();
    let deserialized = from_slice::<Program>(&serialized).unwrap();
    println!("Serialized: {:?}", serialized);
    assert!(
        deserialized == program,
        "Deserialized program was not equivalent to the initial program."
    )
}
