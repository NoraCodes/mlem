use std::io::{Read, Write};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use types::*;
use instructions::*;

/// Represents the outcome of a program run;
/// a halt (graceful termination) or a 
/// fault (hardware error), or a state of continuation,
/// in which the computer can keep running.

#[derive(PartialEq, Debug, Clone)]
pub enum Outcome {
    /// The program halted successfully.
    Halt,
    /// The program caused a problem and broke the machine.
    Fault(String),
    /// The program can continue running.
    Continue
}

/// Represents the state of a machine, including its registers, its memory,
/// its I/O Read and Write, and its program. The associated lifetime `'mach`
/// represents the life of the machine; its I/O connections must live at 
/// least that long.
pub struct Machine<'mach> {
    /// The amount of memory the machine can use, at maximum.
    max_words: usize,
    /// The eight general purpouse registers, used for program operation.
    registers: [Word; 8],
    /// The stack pointer
    sp: Word,
    /// The base pointer
    bp: Word,
    /// The instruction pointer. Note that this is a pointer into the program vector, not
    /// the machine's data memory! It indexes a vector and does NOT advance by bytes or words.
    ip: usize,
    /// Memory used by the machine
    memory: Vec<Word>,
    /// Program code for the machine
    program: Vec<Instruction>,
    /// A reader to get input for the machine
    input: &'mach mut Read,
    /// A writer into which to put output from the machine
    output: &'mach mut Write
}

impl <'mach> Machine <'mach> {
    /// Create a new Machine connected to the given I/O ports.
    pub fn new(max_words: usize, input: &'mach mut Read, output: &'mach mut Write) -> Self {
        Self {
            max_words: max_words,
            registers: [0; 8],
            sp: 0,
            bp: 0,
            ip: 0,
            memory: Vec::new(),
            program: vec![Instruction::Illegal],
            input: input,
            output: output,
        }
    }

    /// Load a program into the machine
    /// This resets the instruction pointer.
    pub fn load_program(&mut self, new: Vec<Instruction>) {
        self.program = new;
        self.ip = 0;
    }

    /// Borrow out the machine's internal memory for examination.
    /// When it's borrowed out, the machine can't run.
    pub fn get_memory(&self) -> &[Word] { &self.memory }

    /// Replace the machine's memory with the given vector.
    pub fn load_memory(&mut self, new: Vec<Word>) { self.memory = new; }

    /// Advance to the next instruction (i.e., increment IP). This can cause a Fault, if IP ends up off the end.
    pub fn next_instr(&mut self) -> Outcome {
        self.ip += 1;
        if self.ip >= self.program.len() {
            Outcome::Fault(format!("IP beyond program length. IP = {}, length = {}", self.ip, self.program.len()))
        } else {
            Outcome::Continue
        }
    }
    
    /// Write the given word value to the given address.
    /// If it's a Literal, this will emit a Fault;
    /// otherwise it's a Continue.
    pub fn write_addr(&mut self, a: Address, v: Word) -> Outcome {
        use self::Address::*;
        match a {
            Literal(l) => { Outcome::Fault(
                    format!("Tried to write {} to literal {}.", v, l)) },
            RegAbs(r) => { self.write_register(r, v); Outcome::Continue  },
            MemAbs(l) => { self.write_memory(l, v) },
            MemReg(r) => { unimplemented!() }
        }
    }

    /// Read a word from the given address.
    pub fn read_addr(&self, a: Address) -> Word {
        use self::Address::*;
        match a {
            Literal(v) => { v },
            RegAbs(r) => { self.read_register(r) },
            MemAbs(l) => { self.read_memory(l) },
            MemReg(r) => {unimplemented!()}
        }
    }

    /// Read a value from a register
    fn read_register(&self, r: Register) -> Word {
        match r {
            Register::R0 => {self.registers[0]},
            Register::R1 => {self.registers[1]},
            Register::R2 => {self.registers[2]},
            Register::R3 => {self.registers[3]},
            Register::R4 => {self.registers[4]},
            Register::R5 => {self.registers[5]},
            Register::R6 => {self.registers[6]},
            Register::R7 => {self.registers[7]},
            Register::SP => {self.sp},
            Register::BP => {self.bp},
        }
    }

    /// Write a value into a register
    fn write_register(&mut self, r: Register, v: Word) {
        match r {
            Register::R0 => {self.registers[0] = v;},
            Register::R1 => {self.registers[1] = v;},
            Register::R2 => {self.registers[2] = v;},
            Register::R3 => {self.registers[3] = v;},
            Register::R4 => {self.registers[4] = v;},
            Register::R5 => {self.registers[5] = v;},
            Register::R6 => {self.registers[6] = v;},
            Register::R7 => {self.registers[7] = v;},
            Register::SP => {self.sp = v;},
            Register::BP => {self.bp = v;},
        }
    }

    /// Write the provided value (v) into the provided memory address.
    /// If this is off the end of the provided memory, fault.
    fn write_memory(&mut self, l: Word, v: Word) -> Outcome {
        let l = l as usize;
        // Memory must be at least the right length
        if l > self.max_words { return Outcome::Fault(
                format!("Tried to write out of available memory: {}", l)); }
        // OK, within the provided memory. Resize if needed.
        if l > self.memory.len() { self.memory.resize(l as usize, 0); }
        self.memory[l] = v;
        Outcome::Continue
    }
    
    /// Read a Word from the provided memory address.
    /// If this address is outsize of the provided memory, this returns 0.
    fn read_memory(&self, l: Word) -> Word {
        let l = l as usize;
        // If it falls outside memory, just give back the default
        if l > self.max_words { 0 }
        else if l > self.memory.len() { 0 }
        else { self.memory[l] }
    }

    pub fn execute_next(&mut self) -> Outcome {
        use Instruction::*;
        // This index operation is safe because next_instr faults if IP goes over the 
        // end of the vector
        match self.program[self.ip] {
            NoOp => { self.ins_no_op() },   
            Zero(a) => { self.ins_zero(a) },         
            Move(a, b) => { self.ins_move(a, b) },
            Output(a) => { self.ins_output(a) },
            Input(a) => { self.ins_input(a) },
            Halt => { self.ins_halt() },
            Illegal => { Outcome::Fault("Illegal instruction encountered.".into()) },
        }
    }

    /// Execute instructions until a Halt or Fault occurs.
    /// _BEWARE: This may run forever!_
    pub fn run(&mut self) -> Outcome {
        loop {
            match self.execute_next() {
                Outcome::Continue => { continue; },
                other => { return other; }
            }
        }
    }

    /// Execute at most the given number of instructions, also stopping on a Halt or Fault condition.
    /// Returns the Outcome of the last instruction and the number of instructions executed.
    pub fn run_for(&mut self, cycles: u64) -> (Outcome, u64) {
        let mut instructions_remaining = cycles;
        while instructions_remaining > 0 {
            match self.execute_next() {
                Outcome::Continue => { instructions_remaining -= 1; },
                other => { return (other, cycles - instructions_remaining); }
            }
        }
        (Outcome::Continue, cycles - instructions_remaining)
    }

    /// Execute a NoOp instruction
    fn ins_no_op(&mut self) -> Outcome {
        self.next_instr()
    }

    /// Execute a Halt instruction
    fn ins_halt(&mut self) -> Outcome { Outcome::Halt }
    
    /// Execute a Move instruction
    fn ins_move(&mut self, a: Address, b: Address) -> Outcome {
        let v = self.read_addr(a);
        match self.write_addr(b, v) {
            Outcome::Continue => { self.next_instr() },
            o => o
       }
    }
    
    /// Execute a Zero instruction
    fn ins_zero(&mut self, a: Address) -> Outcome {
        match self.write_addr(a, 0) {
            Outcome::Continue => { self.next_instr() },
            o => o
       }
    }
    
    /// Execute an Output instruction
    fn ins_output(&mut self, a: Address) -> Outcome {
        let v = self.read_addr(a);
        match self.output.write_u64::<BigEndian>(v) {
            Ok(_) => { self.next_instr() }
            Err(e) => { Outcome::Fault(format!("Failed to write on output instruction: {}.", e)) }
        }
    }

    /// Execute an Input instruction
    fn ins_input(&mut self, a: Address) -> Outcome {
        match self.input.read_u64::<BigEndian>() {
            Ok(v) => {
                match self.write_addr(a, v) {
                    Outcome::Continue => { self.next_instr() },
                    o => o
                }
            },
            Err(e) => { 
            Outcome::Fault(format!("Failed to read on input instruction: {}.", e))
            }
        }
    }
}
