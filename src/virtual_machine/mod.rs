//! A virtual machine capable of executing MLeM in-memory representation.
use crate::*;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
#[cfg(test)]
mod test_machine;

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
    Continue,
}

/// Represents the state of a machine, including its registers, its memory,
/// its I/O Read and Write, and its program.
///
/// The associated lifetime `'mach`
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
    program: Program,
    /// A reader to get input for the machine
    input: &'mach mut Read,
    /// A writer into which to put output from the machine
    output: &'mach mut Write,
}

impl<'mach> Machine<'mach> {
    /// Create a new Machine connected to the given I/O ports.
    pub fn new(max_words: usize, input: &'mach mut Read, output: &'mach mut Write) -> Self {
        Self {
            max_words: max_words,
            registers: [0; 8],
            // Both SP and BP start at the top of memory; the stack grows downwards.
            sp: (max_words - 1) as u64,
            bp: (max_words - 1) as u64,
            ip: 0,
            memory: Vec::with_capacity(max_words),
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
    pub fn get_memory(&self) -> &[Word] {
        &self.memory
    }

    /// Replace the machine's memory with the given vector.
    pub fn load_memory(&mut self, new: Vec<Word>) {
        self.memory = new;
    }

    /// Advance to the next instruction (i.e., increment IP). This can cause a Fault, if IP ends up off the end.
    pub fn next_instr(&mut self) -> Outcome {
        self.ip += 1;
        if self.ip >= self.program.len() {
            Outcome::Fault(format!(
                "IP beyond program length. IP = {}, length = {}",
                self.ip,
                self.program.len()
            ))
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
            Literal(l) => Outcome::Fault(format!("Tried to write {} to literal {}.", v, l)),
            RegAbs(r) => {
                self.write_register(r, v);
                Outcome::Continue
            }
            MemAbs(l) => self.write_memory(l, v),
            MemReg(r) => {
                let location = self.read_register(r);
                self.write_memory(location, v)
            }
        }
    }

    /// Read a word from the given address.
    pub fn read_addr(&self, a: Address) -> Word {
        use self::Address::*;
        match a {
            Literal(v) => v,
            RegAbs(r) => self.read_register(r),
            MemAbs(l) => self.read_memory(l),
            MemReg(r) => self.read_memory(self.read_register(r)),
        }
    }

    /// Read a value from a register
    fn read_register(&self, r: Register) -> Word {
        match r {
            Register::R0 => self.registers[0],
            Register::R1 => self.registers[1],
            Register::R2 => self.registers[2],
            Register::R3 => self.registers[3],
            Register::R4 => self.registers[4],
            Register::R5 => self.registers[5],
            Register::R6 => self.registers[6],
            Register::R7 => self.registers[7],
            Register::SP => self.sp,
            Register::BP => self.bp,
        }
    }

    /// Write a value into a register
    fn write_register(&mut self, r: Register, v: Word) {
        match r {
            Register::R0 => {
                self.registers[0] = v;
            }
            Register::R1 => {
                self.registers[1] = v;
            }
            Register::R2 => {
                self.registers[2] = v;
            }
            Register::R3 => {
                self.registers[3] = v;
            }
            Register::R4 => {
                self.registers[4] = v;
            }
            Register::R5 => {
                self.registers[5] = v;
            }
            Register::R6 => {
                self.registers[6] = v;
            }
            Register::R7 => {
                self.registers[7] = v;
            }
            Register::SP => {
                self.sp = v;
            }
            Register::BP => {
                self.bp = v;
            }
        }
    }

    /// Write the provided value (v) into the provided memory address.
    /// If this is off the end of the provided memory, fault.
    fn write_memory(&mut self, l: Word, v: Word) -> Outcome {
        let l = l as usize;
        // Memory must be at least the right length
        if l > self.max_words {
            return Outcome::Fault(format!("Tried to write out of available memory: {}", l));
        }
        // OK, within the provided memory. Resize if needed.
        if l > self.memory.len() {
            self.memory.resize(l + 1 as usize, 0);
        }
        self.memory[l] = v;
        Outcome::Continue
    }

    /// Read a Word from the provided memory address.
    /// If this address is outsize of the provided memory, this returns 0.
    fn read_memory(&self, l: Word) -> Word {
        let l = l as usize;
        // If it falls outside memory, just give back the default
        if l > self.max_words {
            0
        } else if l > self.memory.len() {
            0
        } else {
            self.memory[l]
        }
    }

    fn absolute_jump(&mut self, l: JumpLocation) -> Outcome {
        if l < self.program.len() {
            self.ip = l;
            Outcome::Continue
        } else {
            Outcome::Fault(format!(
                "Attempt to jump to {} would overrun program of length {}.",
                l,
                self.program.len()
            ))
        }
    }

    pub fn execute_next(&mut self) -> Outcome {
        use Instruction::*;
        // This index operation is safe because next_instr faults if IP goes over the
        // end of the vector
        match self.program[self.ip] {
            NoOp => self.ins_no_op(),
            Zero(a) => self.ins_zero(a),
            Move(a, b) => self.ins_move(a, b),
            Output(a) => self.ins_output(a),
            Input(a) => self.ins_input(a),
            Add(a, b) => self.ins_generic_scalar(a, b, |va, vb| va.wrapping_add(vb)),
            Sub(a, b) => self.ins_generic_scalar(a, b, |va, vb| va.wrapping_sub(vb)),
            Jump(a) => self.ins_jump(a),
            JumpIfZero(a, b) => self.ins_generic_jump_single(a, b, |v| v == 0),
            JumpNotZero(a, b) => self.ins_generic_jump_single(a, b, |v| v != 0),
            Push(a) => self.ins_push(a),
            Pop(a) => self.ins_pop(a),
            Halt => self.ins_halt(),
            Illegal => Outcome::Fault("Illegal instruction encountered.".into()),
        }
    }

    /// Execute instructions until a Halt or Fault occurs.
    /// _BEWARE: This may run forever!_
    pub fn run(&mut self) -> Outcome {
        loop {
            match self.execute_next() {
                Outcome::Continue => {}
                other => {
                    return other;
                }
            }
        }
    }

    /// Execute at most the given number of instructions, also stopping on a Halt or Fault condition.
    /// Returns the Outcome of the last instruction and the number of instructions executed.
    pub fn run_for(&mut self, cycles: u64) -> (Outcome, u64) {
        let mut instructions_remaining = cycles;
        while instructions_remaining > 0 {
            match self.execute_next() {
                Outcome::Continue => {
                    instructions_remaining -= 1;
                }
                other => {
                    return (other, cycles - instructions_remaining);
                }
            }
        }
        (Outcome::Continue, cycles - instructions_remaining)
    }

    /// Execute a NoOp instruction
    fn ins_no_op(&mut self) -> Outcome {
        self.next_instr()
    }

    /// Execute a Halt instruction
    fn ins_halt(&mut self) -> Outcome {
        Outcome::Halt
    }

    /// Execute a Move instruction
    fn ins_move(&mut self, a: Address, b: Address) -> Outcome {
        let v = self.read_addr(a);
        match self.write_addr(b, v) {
            Outcome::Continue => self.next_instr(),
            o => o,
        }
    }

    /// Execute a Zero instruction
    fn ins_zero(&mut self, a: Address) -> Outcome {
        match self.write_addr(a, 0) {
            Outcome::Continue => self.next_instr(),
            o => o,
        }
    }

    /// Execute an Output instruction
    fn ins_output(&mut self, a: Address) -> Outcome {
        let v = self.read_addr(a);
        match self.output.write_u64::<BigEndian>(v) {
            Ok(_) => self.next_instr(),
            Err(e) => Outcome::Fault(format!("Failed to write on output instruction: {}.", e)),
        }
    }

    /// Execute an Input instruction
    fn ins_input(&mut self, a: Address) -> Outcome {
        match self.input.read_u64::<BigEndian>() {
            Ok(v) => match self.write_addr(a, v) {
                Outcome::Continue => self.next_instr(),
                o => o,
            },
            Err(e) => Outcome::Fault(format!("Failed to read on input instruction: {}.", e)),
        }
    }

    /// Execute any 2-register scalar instruction
    fn ins_generic_scalar<F: FnOnce(Word, Word) -> Word>(
        &mut self,
        a: Address,
        b: Address,
        f: F,
    ) -> Outcome {
        let value_a = self.read_addr(a);
        let value_b = self.read_addr(b);
        match self.write_addr(a, f(value_a, value_b)) {
            Outcome::Continue => self.next_instr(),
            other => other,
        }
    }

    /// Execute an unconditional jump
    fn ins_jump(&mut self, a: Address) -> Outcome {
        let addr = self.read_addr(a) as JumpLocation;
        self.absolute_jump(addr)
    }

    /// Execute any one-operand jump
    fn ins_generic_jump_single<F: FnOnce(Word) -> bool>(
        &mut self,
        a: Address,
        b: Address,
        f: F,
    ) -> Outcome {
        let value_a = self.read_addr(a) as JumpLocation;
        let value_b = self.read_addr(b);
        if f(value_b) {
            self.absolute_jump(value_a)
        } else {
            self.next_instr()
        }
    }

    /// Execute a push instruction. Causes a fault if the stack has overrun the available
    /// memory.
    fn ins_push(&mut self, a: Address) -> Outcome {
        let val = self.read_addr(a);
        // Scope for mutable borrow
        self.sp -= 1;
        if self.sp <= 0 {
            Outcome::Fault("Stack has overrun available memory!".into())
        } else {
            // Copy out of immutable ref to self to satisfy borrow checker
            let location = self.sp;
            self.write_memory(location, val);
            self.next_instr()
        }
    }

    /// Execute a pop instruction. If the stack is empty, this does not fault, but sets the target to
    /// zero.
    fn ins_pop(&mut self, a: Address) -> Outcome {
        let val = if self.sp >= self.bp {
            self.sp = self.bp;
            0
        } else {
            self.read_memory(self.sp)
        };
        self.sp += 1;

        match self.write_addr(a, val) {
            Outcome::Continue => self.next_instr(),
            other => other,
        }
    }
}

/// Given a Program (that is, a Vec of Instructions), this function will manage creating a Machine and hooking up its
/// Input and Output for you. It returns a tuple of the final outcome of the program, the number of instructions executed, and
/// a Vector of the output.
pub fn execute(program: Program, input: Vec<u64>, limit: Option<u64>) -> (Outcome, u64, Vec<u64>) {
    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
    use std::io::{Cursor, Seek};
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
        let actual_limit = limit.unwrap_or(u64::max_value());
        let (a, b) = m.run_for(actual_limit);
        o = a;
        cycles = b;
    }
    // Compose output into u64 values
    let mut output = Vec::new();
    internal_output.seek(std::io::SeekFrom::Start(0)).unwrap();
    while let Ok(v) = internal_output.read_u64::<BigEndian>() {
        output.push(v);
    }

    (o, cycles, output)
}
