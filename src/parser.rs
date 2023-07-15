use crate::instructions::{help, Instruction, StackType};
use crate::runner::Runner;
use num_complex::Complex;
use std::collections::HashMap;

pub struct Parser {
    verbose: bool,
    runner: Runner,
    instructions: Vec<Instruction>,
    last_number: StackType,                // for real, imag
    procedure_lut: HashMap<String, usize>, // for the parser
    procedure_state: u8,
    loop_addr: Vec<usize>,
}

impl Parser {
    pub fn new(verbose: bool) -> Self {
        Parser {
            verbose,
            runner: Runner::new(verbose),
            instructions: vec![],

            last_number: StackType::None,
            procedure_lut: HashMap::new(),
            procedure_state: 0,
            loop_addr: vec![],
        }
    }

    fn get_reg(&mut self) -> Option<u8> {
        let StackType::Double(a) = self.last_number else {eprintln!("Register number needed before this instruction."); return None};
        self.last_number = StackType::None;
        Some(a as u8)
    }

    pub fn parse_line(&mut self, line: &str) {
        for token in line.split('#').next().unwrap().split_whitespace() {
            if self.verbose {
                println!("Debug: parser token: {token}");
            }
            let mut last_command_not_parse_double = true;
            match token {
                // Stack operations
                "dup" => self.instructions.push(Instruction::Dup),
                "drop" => self.instructions.push(Instruction::Drop),
                "over" => self.instructions.push(Instruction::Over),
                "rot" => self.instructions.push(Instruction::Rot),
                "swap" => self.instructions.push(Instruction::Swap),
                "clear" => self.instructions.push(Instruction::Clear),
                "dumpstack" | "ds" => self.instructions.push(Instruction::DumpStack),

                // Basic arithmetic
                "+" | "add" => self.instructions.push(Instruction::Add),
                "-" | "sub" => self.instructions.push(Instruction::Sub),
                "*" | "mul" => self.instructions.push(Instruction::Mul),
                "/" | "div" => self.instructions.push(Instruction::Div),
                "and" => self.instructions.push(Instruction::And),
                "or" => self.instructions.push(Instruction::Or),
                "xor" => self.instructions.push(Instruction::Xor),
                "neg" => self.instructions.push(Instruction::Neg),
                "shl" => self.instructions.push(Instruction::Shl),
                "shr" => self.instructions.push(Instruction::Shr),
                "abs" => self.instructions.push(Instruction::Abs),
                "floor" => self.instructions.push(Instruction::Floor),
                "ceil" => self.instructions.push(Instruction::Ceil),
                "round" => self.instructions.push(Instruction::Round),

                // Trigonometric function
                "cosr" => self.instructions.push(Instruction::CosR),
                "sinr" => self.instructions.push(Instruction::SinR),
                "tanr" => self.instructions.push(Instruction::TanR),
                "cosd" => self.instructions.push(Instruction::CosD),
                "sind" => self.instructions.push(Instruction::SinD),
                "tand" => self.instructions.push(Instruction::TanD),
                "acosr" => self.instructions.push(Instruction::AcosR),
                "asinr" => self.instructions.push(Instruction::AsinR),
                "atanr" => self.instructions.push(Instruction::AtanR),
                "acosd" => self.instructions.push(Instruction::AcosD),
                "asind" => self.instructions.push(Instruction::AsinD),
                "atand" => self.instructions.push(Instruction::AtanD),

                // Logarithm and exponential
                "loge" => self.instructions.push(Instruction::Loge),
                "log2" => self.instructions.push(Instruction::Log2),
                "log10" => self.instructions.push(Instruction::Log10),
                "logx" => self.instructions.push(Instruction::Logx),
                "expe" => self.instructions.push(Instruction::Expe),
                "exp2" => self.instructions.push(Instruction::Exp2),
                "exp10" => self.instructions.push(Instruction::Exp10),
                "expx" => self.instructions.push(Instruction::Expx),
                ">" => self.instructions.push(Instruction::Gt),
                "<" => self.instructions.push(Instruction::Lt),
                ">=" => self.instructions.push(Instruction::Ge),
                "<=" => self.instructions.push(Instruction::Le),
                "=" => self.instructions.push(Instruction::Eq),

                // Complex
                "real" => self.instructions.push(Instruction::Real),
                "imag" => self.instructions.push(Instruction::Imag),
                "r2c" => self.instructions.push(Instruction::R2c),

                // Print and related
                "k" | "precision" => self.instructions.push(Instruction::Precision),
                "K" => self.instructions.push(Instruction::GetPrecision),
                "p" | "print" => self.instructions.push(Instruction::Print),

                // Register
                "save" => {
                    let Some(reg) = self.get_reg() else {break};
                    self.instructions.push(Instruction::Save(reg));
                }
                "load" => {
                    let Some(reg) = self.get_reg() else {break};
                    self.instructions.push(Instruction::Load(reg));
                }
                "creg" => {
                    let Some(reg) = self.get_reg() else {break};
                    self.instructions.push(Instruction::Creg(reg));
                }
                "clregs" => self.instructions.push(Instruction::Clregs),
                "dumpreg" | "dr" => self.instructions.push(Instruction::DumpReg),

                // Vector
                "vreal" => {
                    let Some(reg) = self.get_reg() else {break};
                    self.instructions.push(Instruction::Vreal(reg));
                }
                "vcplx" => {
                    let Some(reg) = self.get_reg() else {break};
                    self.instructions.push(Instruction::Vcplx(reg));
                }
                "vsave" => {
                    let Some(reg) = self.get_reg() else {break};
                    self.instructions.push(Instruction::Vsave(reg));
                }
                "vload" => {
                    let Some(reg) = self.get_reg() else {break};
                    self.instructions.push(Instruction::Vload(reg));
                }
                "dumpvec" | "dv" => self.instructions.push(Instruction::DumpVec),

                // Procedure and loop:
                ":" => {
                    self.runner.run(&self.instructions);
                    self.instructions.clear();
                    self.procedure_state = 1;
                }
                ";" => {
                    self.instructions.push(Instruction::Ret);
                    self.runner.add_instr(&self.instructions);
                    self.instructions.clear();
                    self.procedure_state = 0;
                }
                "[" => self
                    .loop_addr
                    .push(self.runner.get_proglen() + self.instructions.len()),
                "]" => self
                    .instructions
                    .push(Instruction::Jnz(self.loop_addr.pop().unwrap())),

                // Interpreter direct func
                "help" => {
                    help();
                }
                "quit" | "bye" | "exit" | "q" => {
                    self.instructions.push(Instruction::Quit);
                }
                _ => {
                    if self.procedure_state == 1 {
                        self.procedure_lut.insert(
                            token.to_string(),
                            self.runner.get_proglen() + self.instructions.len(),
                        );
                        self.procedure_state = 2;
                    } else if let Some(&call_ptr) = self.procedure_lut.get(token) {
                        // token -> call subrutin
                        self.instructions.push(Instruction::Call(call_ptr));
                    } else if token.as_bytes()[0].is_ascii_digit() || token.as_bytes()[0] == b'-' {
                        // Possible number (real or imag).
                        // Imag check --> 4.32j
                        if token.as_bytes().last().unwrap() == &b'j' {
                            let t2 = &token[0..token.len() - 1];
                            let Ok(imag) = t2.parse::<f64>() else {eprintln!("Number error");break};
                            // if prevous was a normal Double, it is the real part of complex.
                            let cmplx = if let StackType::Double(a) = self.last_number {
                                Complex::new(a, imag)
                            } else {
                                Complex::new(0.0, imag)
                            };
                            self.instructions
                                .push(Instruction::Literal(StackType::Complex(cmplx)));
                            self.last_number = StackType::None;
                        } else {
                            // Double or real part ... if prevous was a normal Double, write
                            if let StackType::Double(a) = self.last_number {
                                self.instructions
                                    .push(Instruction::Literal(StackType::Double(a)));
                            }
                            let Ok(number) = token.parse::<f64>() else {eprintln!("Number error");break};
                            self.last_number = StackType::Double(number);
                            last_command_not_parse_double = false;
                        }
                    } else {
                        eprintln!("Not a number, invalid command. Please type 'help'.");
                    }
                }
            } // match

            // if the number storeable - does not have imaginary part
            if last_command_not_parse_double {
                if let StackType::Double(_) = self.last_number {
                    if !self.instructions.is_empty() {
                        let last_instr = self.instructions.pop().unwrap();
                        self.instructions
                            .push(Instruction::Literal(self.last_number));
                        self.instructions.push(last_instr);
                    } else {
                        self.instructions
                            .push(Instruction::Literal(self.last_number));
                    }
                    self.last_number = StackType::None;
                }
            }
        } // for token
        if self.procedure_state == 0 && !self.instructions.is_empty() {
            self.runner.run(&self.instructions);
            self.instructions.clear();
        }
    } // end fn parse
} // end Parse
