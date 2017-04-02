use std::io::{Cursor, Seek};
use std::rc::Rc;
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

    // Load a program with just the one instruction
    m.load_program(vec![instr]);

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

        // Load a program with just the one instruction
        m.load_program(vec![instr]);

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
