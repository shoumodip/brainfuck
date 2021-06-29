use std::fs;
use std::env;
use std::process;
use std::io::{stdout, stdin, Read, Write};

#[derive(Debug)]
enum Inst {
    Inc,
    Dec,
    Input,
    Output,
    ShiftLeft,
    ShiftRight,
    Goto(usize)
}

/// Compile a BF program to its OpCode representation
fn compile(file_path: &str) -> Vec<Inst> {
    let source = match fs::read_to_string(file_path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error: failed to read file `{}`: {}", file_path, e);
            process::exit(1);
        }
    };

    let mut output = vec![];
    let mut loops = vec![];
    let mut index = 0;

    let mut line = 1;
    let mut column = 0;

    use Inst::*;
    for c in source.chars() {
        column += 1;

        match c {
            '+' => output.push(Inc),
            '-' => output.push(Dec),
            ',' => output.push(Input),
            '.' => output.push(Output),
            '<' => output.push(ShiftLeft),
            '>' => output.push(ShiftRight),
            '[' => {
                loops.push(if index == 0 {0} else {index - 1});
                continue;
            },
            ']' => match loops.pop() {
                Some(0) => {
                    // Loop at the start of the program is a guaranted comment
                    index = 0;
                    output.clear();
                    continue;
                },
                Some(i) => output.push(Goto(i)),
                None => {
                    eprintln!("{}:{}:{} Unbalanced ']'", file_path, line, column);
                    process::exit(1);
                }
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

    if !loops.is_empty() {
        eprintln!("{}:{}:{} Unbalanced '['", file_path, line, column);
        process::exit(1);
    }

    output
}

/// The length of the tape
const TAPE_LENGTH: usize = 30000;

/// The virtual machine where the program is executed
struct Vm {
    memory: [u8; TAPE_LENGTH],
    mp: usize,
    ip: usize,
    program: Vec<Inst>
}

impl Vm {

    /// Create a virtual machine from a source program
    fn new(program: Vec<Inst>) -> Self {
        Self {
            memory: [0; TAPE_LENGTH],
            mp: 0,
            ip: 0,
            program
        }
    }

    /// Execute the current instruction
    fn execute(&mut self) {
        use Inst::*;

        match self.program[self.ip] {
            ShiftLeft => self.mp = if self.mp == 0 {TAPE_LENGTH - 1} else {self.mp - 1},
            ShiftRight => self.mp = if self.mp == TAPE_LENGTH - 1 {0} else {self.mp + 1},

            Inc => self.memory[self.mp] = if self.memory[self.mp] == u8::MAX {
                0
            } else {
                self.memory[self.mp] + 1
            },

            Dec => self.memory[self.mp] = if self.memory[self.mp] == 0 {
                u8::MAX
            } else {
                self.memory[self.mp] - 1
            },

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

            Goto(i) => {
                if self.memory[self.mp] != 0 {
                    self.ip = i;
                }
            }
        }
    }

    /// Start the virtual machine
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
