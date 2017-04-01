use std::io::{Read, Write};
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

/// Represents the state of a machine.
pub struct Machine {
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
    input: Box<Read>,
    /// A writer into which to put output from the machine
    output: Box<Write>
}

impl Machine {
    /// Create a new Machine connected to the given I/O ports.
    pub fn new(max_words: usize, input: Box<Read>, output: Box<Write>) -> Self {
        Self {
            max_words: max_words,
            registers: [0; 8],
            sp: 0,
            bp: 0,
            ip: 0,
            memory: Vec::new(),
            program: Vec::new(),
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

    pub fn execute_next(&mut self) -> Outcome {
        use Instruction::*;
        // This index operation is safe because next_instr faults if IP goes over the 
        // end of the vector
        match self.program[self.ip] {
            NoOp => { self.ins_no_op() },            
            Move(a, b) => { self.ins_move(a, b) },
            _ => { unimplemented!() } 
        }
    }

    /// Advance to the next instruction.
    pub fn next_instr(&mut self) -> Outcome {
        self.ip += 1;
        if self.ip > self.program.len() {
            Outcome::Fault(format!("IP beyond program length: {}", self.ip))
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

    /// Execute a NoOp instruction
    fn ins_no_op(&mut self) -> Outcome {
        self.next_instr()
    }
    
    /// Execute a Move instruction
    fn ins_move(&mut self, a: Address, b: Address) -> Outcome {
        let v = self.read_addr(a);
        self.write_addr(b, v)
    }
}
