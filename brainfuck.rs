use std::fs;
use std::env;
use std::process;
use std::io::{stdout, stdin, Read, Write};

// Instructions for the VM
enum Inst {
    Inc(usize),
    Dec(usize),
    ShiftRight(usize),
    ShiftLeft(usize),
    Input(usize),
    Output(usize),
    LoopStart(usize),
    LoopEnd(usize),
}

// Print a message to standard error and exit
fn error(message: &str) {
    eprintln!("{}", message);
    process::exit(1);
}

// If the last instruction in the "bytecode" is of the same type as
// the one to be appended, then the amount of the last instruction is
// increased instead
macro_rules! sized_inst {
    ($output: expr, $index: expr, $type: tt) => {{
        if let Some($type(n)) = $output.last() {
            $output[$index - 1] = $type(n + 1);
            continue;
        } else {
            $output.push($type(1));
        }
    }};
}

// Compile a BF program to an instruction chunk for the turing machine
fn compile(file_path: &str) -> Vec<Inst> {
    let source = fs::read_to_string(file_path).unwrap_or_else(|_| {
        eprintln!("error: failed to read file '{}'", file_path);
        process::exit(1);       // Unreachable
    });

    let mut output = vec![];
    let mut loops = vec![];
    let mut index = 0;

    let mut line = 1;
    let mut column = 0;

    use Inst::*;
    for c in source.chars() {
        column += 1;

        match c {
            '+' => sized_inst!(output, index, Inc),
            '-' => sized_inst!(output, index, Dec),
            '>' => sized_inst!(output, index, ShiftRight),
            '<' => sized_inst!(output, index, ShiftLeft),
            ',' => sized_inst!(output, index, Input),
            '.' => sized_inst!(output, index, Output),
            '[' => {
                loops.push((index, line, column));
                output.push(LoopStart(index));
            },
            ']' => match loops.pop() {
                Some((0, _, _)) => {
                    // Loop at the start of the program is a guaranted comment
                    index = 0;
                    loops.clear();
                    output.clear();
                    continue;
                },
                Some((i, _, _)) => {
                    output[i] = LoopStart(index);
                    output.push(LoopEnd(i));
                },
                None => error(&format!("{}:{}:{} Unbalanced ']'",
                                       file_path, line, column))
            },
            '\n' => {
                line += 1;
                column = 0;
                continue;
            },
            _ => continue
        }

        index += 1;
    }

    if let Some((_, line, column)) = loops.last() {
        error(&format!("{}:{}:{}: Unterminated '['",
                       file_path, line, column));
    }

    output
}

// The length of the tape
const TAPE_LENGTH: usize = 30000;

// The virtual machine where the program is executed
struct Vm {
    memory: [u8; TAPE_LENGTH],
    mp: usize,
    ip: usize,
    program: Vec<Inst>
}

// Wrap around the edges in a number with customized type annotations
macro_rules! modulo {
    ($value: expr, $limit: expr, $type: tt) => {{
        let limit = $limit as isize + 1;
        let value = $value as isize;

        let value = if value >= limit {
            value % limit
        } else if value < 0 {
            limit - isize::abs(value) % limit
        } else {
            value
        };

        value as $type
    }};
}

impl Vm {
    // Create a virtual machine from a source program
    fn new(program: Vec<Inst>) -> Self {
        Self {
            memory: [0; TAPE_LENGTH],
            mp: 0,
            ip: 0,
            program
        }
    }

    // Execute the current instruction
    fn execute(&mut self) {
        use Inst::*;

        match self.program[self.ip] {
            Inc(amount) => self.memory[self.mp] = modulo!(self.memory[self.mp] as isize + amount as isize, u8::MAX, u8),
            Dec(amount) => self.memory[self.mp] = modulo!(self.memory[self.mp] as isize - amount as isize, u8::MAX, u8),

            ShiftRight(amount) => self.mp = modulo!(self.mp as isize + amount as isize, TAPE_LENGTH, usize),
            ShiftLeft(amount) => self.mp = modulo!(self.mp as isize - amount as isize, TAPE_LENGTH, usize),

            Output => {
                print!("{}", self.memory[self.mp] as char);
                stdout().flush().expect("brainfuck: Failed to flush stdout");
            },

            Input => {
                self.memory[self.mp] = stdin()
                    .bytes()
                    .next()
                    .expect("brainfuck: Failed to read from stdin")
                    .expect("brainfuck: Failed to read from stdin");
            },

            LoopStart(i) => {
                if self.memory[self.mp] == 0 {
                    self.ip = i;
                }
            },

            LoopEnd(i) => {
                if self.memory[self.mp] != 0 {
                    self.ip = i;
                }
            }
        }
    }

    // Start the virtual machine
    fn start(&mut self) {
        while self.ip < self.program.len() {
            self.execute();
            self.ip += 1;
        }
    }
}

fn main() {
    let mut files = 0;

    for (index, file_path) in env::args().enumerate() {
        if index == 0 { continue; }

        let mut vm = Vm::new(compile(&file_path));
        vm.start();

        files += 1;
    }

    if files == 0 {
        eprintln!("error: No input files were provided");
        eprintln!("Usage: brainfuck [FILE-1] [...]");
        process::exit(1);
    }
}
