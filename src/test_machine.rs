use std::io::Cursor;
use super::*;
fn make_test_machine() -> Machine {
    let input:  Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let output: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    Machine::new(128, Box::new(input), Box::new(output))

}

#[test]
fn test_get_set_memory() {
    let mut m = make_test_machine();
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
    let mut m = make_test_machine();
    let instr = Instruction::Move(
        Address::Literal(0xDEADBEEF),
        Address::RegAbs(Register::R0)
    );
    // Load a program with just the one instruction
    m.load_program(vec![instr]);
    // Run one instruction.
    let o = m.execute_next();
    assert!(Outcome::Continue == o, "Move instruction caused {:?}.", o);
}
