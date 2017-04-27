use std::io::{Cursor, Seek};
use byteorder::{BigEndian, ReadBytesExt};
use super::*;

#[test]
fn test_get_set_memory() {
    let mut input:  Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(128));
    let mut output: Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(128));
    let mut m = Machine::new(128, &mut input, &mut output);

    let memory = vec![0, 1, 2, 3, 4];
    let mem_copy = memory.clone();

    m.load_memory(memory);
    assert!(&mem_copy as &[u64] == m.get_memory(), 
            "Machine's returned memory was not the same as the loaded memory.");

    m.write_addr(Address::MemAbs(0), 0xDEADBEEF);
    assert!(0xDEADBEEF == m.read_addr(Address::MemAbs(0)), 
            "Read memory at MemAbs 0 was not the same as written!");
}

#[test]
fn test_reg_instructions() {
    let mut input:  Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let mut output: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let mut m = Machine::new(128, &mut input, &mut output);

    // The instruction: move 0xdeadbeef R0
    let instr = Instruction::Move(
        Address::Literal(0xDEADBEEF),
        Address::RegAbs(Register::R0)
    );

    // Load a program with just the one instruction and a Halt to prevent Faulting
    m.load_program(vec![instr, Instruction::Halt]);

    // Run one instruction forward
    let o = m.execute_next();
    assert!(Outcome::Continue == o, "Move instruction caused {:?}.", o);
}

#[test]
fn test_io_instructions() {
    let mut input:  Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let mut output: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    // The machine lives inside this block
    {
        let mut m = Machine::new(128, &mut input, &mut output);

        // Create the assembly instruction:
        //  output 0xDEADBEEF
        let instr = Instruction::Output(
            Address::Literal(0xDEADBEEF)
        );

        // Load a program with just the one instruction and a Halt to prevent Faulting
        m.load_program(vec![instr, Instruction::Halt]);

        // Run one instruction.
        let outcome = m.execute_next();
        assert!(Outcome::Continue == outcome, "Move instruction caused {:?}.", outcome);
    }

    // Return to the beginning of the output buffer; the machine will probably have moved the Cursor.
    output.seek(std::io::SeekFrom::Start(0)).unwrap();

    match output.read_u64::<BigEndian>() {
        Ok(v) => assert!( v == 0xDEADBEEF, "Ouput was not the expected {:?} but rather {:?}.", 0xDEADBEEFu64, v),
        Err(e) => panic!("Failed to read from output buffer: {}. Buffer is: {:?}", e, output.get_ref())
    }
}

#[test]
fn test_run() {
    let mut input:  Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(128));
    let mut output: Cursor<Vec<u8>> = Cursor::new(Vec::with_capacity(128));
    let mut m = Machine::new(128, &mut input, &mut output);

    // A 4-instruction program.
    let program = vec![
        Instruction::NoOp,
        Instruction::NoOp,
        Instruction::NoOp,
        Instruction::NoOp,
        Instruction::Halt,
    ];

    m.load_program(program);

    let (limited_outcome, instructions_executed) = m.run_for(3);
    assert!(limited_outcome == Outcome::Continue, "Program produced {:?} after {:?} NoOp instructions.", limited_outcome, instructions_executed);
    assert!(instructions_executed == 3, "Program reports executing {:?} instructions rather than the 3 expected.", instructions_executed);

    let final_outcome = m.run();
    assert!(final_outcome == Outcome::Halt, "Program produced {:?} rather than halting.", final_outcome);
}

#[test]
fn test_scalar_arith() {
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

    let (outcome, _, output) = execute(program, input, Some(10));
    assert!(outcome == Outcome::Halt, "Program did not successfully halt! {:?}", outcome);
    assert!(output == expected, "Program did not produce {:?} as expected, but rather {:?}.", expected, output);
}

#[test]
fn test_jump() {
    let input = vec![1, 2, 3];
    let expected = vec![1, 3];
    // Explaination: Loads input into the first three GPRs, then outputs the same numbers again, but
    // uses a jump instruction to skip outputing the second number
    let program = vec![
        // Get all input values
        Instruction::Input(Address::RegAbs(Register::R0)), // 0
        Instruction::Input(Address::RegAbs(Register::R1)), // 1
        Instruction::Input(Address::RegAbs(Register::R2)), // 2
        // Output computed values
        Instruction::Output(Address::RegAbs(Register::R0)), // 3
        Instruction::Jump(Address::Literal(6)),             // 4
        Instruction::Output(Address::RegAbs(Register::R1)), // 5 GETS SKIPPED
        Instruction::Output(Address::RegAbs(Register::R2)), // 6
        // Halt
        Instruction::Halt
    ];

    let (outcome, _, output) = execute(program, input, Some(10));
    assert!(outcome == Outcome::Halt, "Program did not successfully halt! {:?}", outcome);
    assert!(output == expected, "Program did not produce {:?} as expected, but rather {:?}.", expected, output);
}

#[test]
fn test_conditional_jump() {
    let input = vec![1, 2, 3];
    let expected = vec![1, 2, 3, 1, 2, 3];
    // Explaination: Loads input into the first three GPRs, then outputs the same numbers again, and uses
    // a jump instruction to print them again.
    let program = vec![
        // Set the counter
        Instruction::Move(Address::Literal(2), Address::RegAbs(Register::R7)), // 0
        // Get all input values
        Instruction::Input(Address::RegAbs(Register::R0)), // 1
        Instruction::Input(Address::RegAbs(Register::R1)), // 2
        Instruction::Input(Address::RegAbs(Register::R2)), // 3
        // Output computed values
        Instruction::Output(Address::RegAbs(Register::R0)), // 4
        Instruction::Output(Address::RegAbs(Register::R1)), // 5
        Instruction::Output(Address::RegAbs(Register::R2)), // 6
        // Loop
        Instruction::Sub(Address::RegAbs(Register::R7), Address::Literal(1)), // 7
        Instruction::JumpNotZero(Address::Literal(4), Address::RegAbs(Register::R7)), // 8
        // Halt
        Instruction::Halt // 9

    ];

    let (outcome, _, output) = execute(program, input, Some(30));
    assert!(outcome == Outcome::Halt, "Program did not successfully halt! {:?}", outcome);
    assert!(output == expected, "Program did not produce {:?} as expected, but rather {:?}.", expected, output);
}

#[test]
fn test_stack() {
    let input = vec![1, 2, 3];
    let expected = vec![3, 2, 1, 0];
    // Explaination: Loads input into the stack, then pops it off and outputs it.
    let program = vec![
        // Set the counter
        Instruction::Move(Address::Literal(3), Address::RegAbs(Register::R7)), // 0
        // Get all input values
        Instruction::Input(Address::RegAbs(Register::R0)), // 1
        Instruction::Push(Address::RegAbs(Register::R0)), // 2
        // Loop
        Instruction::Sub(Address::RegAbs(Register::R7), Address::Literal(1)), // 3
        Instruction::JumpNotZero(Address::Literal(1), Address::RegAbs(Register::R7)), // 4
        // Set the counter
        Instruction::Move(Address::Literal(4), Address::RegAbs(Register::R7)), // 0
        // Output computed values
        Instruction::Pop(Address::RegAbs(Register::R0)), // 6
        Instruction::Output(Address::RegAbs(Register::R0)), // 7
        // Loop
        Instruction::Sub(Address::RegAbs(Register::R7), Address::Literal(1)), // 8
        Instruction::JumpNotZero(Address::Literal(6), Address::RegAbs(Register::R7)), // 9
        // Halt
        Instruction::Halt // 10

    ];

    let (outcome, _, output) = execute(program, input, Some(40));
    assert!(outcome == Outcome::Halt, "Program did not successfully halt! {:?}", outcome);
    assert!(output == expected, "Program did not produce {:?} as expected, but rather {:?}.", expected, output);
}

#[test]
fn test_pointer_memory_access() {
    let input = vec![0];
    let expected = vec![0xff];
    // Explaination: Loads input into the stack, then pops it off and outputs it.
    let program = vec![
        // Set R0 = 16
        Instruction::Move(Address::Literal(0xf), Address::RegAbs(Register::R0)),
        // Set mem[0xf] = 0xff
        Instruction::Move(Address::Literal(0xff), Address::MemReg(Register::R0)),
        // Read mem[0xf] and output
        Instruction::Output(Address::MemReg(Register::R0)),
        // Halt
        Instruction::Halt // 10

    ];
    let (outcome, _, output) = execute(program, input, Some(5));
    assert!(outcome == Outcome::Halt, "Program did not successfully halt! {:?}", outcome);
    assert!(output == expected, "Program did not produce {:?} as expected, but rather {:?}.", expected, output);
}