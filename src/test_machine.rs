use std::io::{Cursor, Seek};
use std::rc::Rc;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use super::*;

fn test_prog_io(program: Program, input: Vec<u64>) -> (Outcome, Vec<u64>, u64) {
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
        let (a, b) = m.run_for(1024);
        o = a;
        cycles = b;
    }
    // Compose output into u64 values
    let mut output = Vec::new();
    internal_output.seek(std::io::SeekFrom::Start(0)).unwrap();
    while let Ok(v) = internal_output.read_u64::<BigEndian>() {
        output.push(v);
    }

    (o, output, cycles)
}

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

    let memory = vec![0, 1, 2, 3, 4];
    let mem_copy = memory.clone();

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

    let (outcome, output, _) = test_prog_io(program, input);
    assert!(outcome == Outcome::Halt, "Program did not successfully halt! {:?}", outcome);
    assert!(output == expected, "Program did not produce {:?} as expected, but rather {:?}.", expected, output);
}