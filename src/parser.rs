use crate::instructions::{help, Instruction};
use crate::runner::RealRunner;
use std::collections::HashMap;

pub struct Parser {
    verbose: bool,
    runner: RealRunner,
    instructions: Vec<Instruction>,
    procedure_lut: HashMap<String, (usize, String)>, // for the parser and print description
    procedure_state: u8,
    loop_addr: Vec<usize>,
    complex_mode: bool,
}

impl Parser {
    pub fn new(verbose: bool) -> Self {
        Parser {
            verbose,
            runner: RealRunner::new(verbose),
            instructions: vec![],

            procedure_lut: HashMap::new(),
            procedure_state: 0,
            loop_addr: vec![],
            complex_mode: false,
        }
    }

    fn get_reg(&mut self) -> Option<u8> {
        if let Some(Instruction::Literal(a)) = self.instructions.last() {
            let ret = Some(*a as u8);
            let _ = self.instructions.pop();
            ret
        } else {
            eprintln!("Register number needed before this instruction.");
            None
        }
    }

    pub fn parse_line(&mut self, line: &str) {
        for token in line.split('#').next().unwrap().split_whitespace() {
            if self.verbose {
                println!("Debug: parser token: {token}");
            }
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
                /*
                        "real" => self.instructions.push(Instruction::Real),
                        "imag" => self.instructions.push(Instruction::Imag),
                        "r2c" => self.instructions.push(Instruction::R2c),
                */
                // Print and related
                "frdigit" => self.instructions.push(Instruction::FractionalDigit),
                "p" | "print" => self.instructions.push(Instruction::Print),

                // Register
                "save" => {
                    let Some(reg) = self.get_reg() else { break };
                    self.instructions.push(Instruction::Save(reg));
                }
                "load" => {
                    let Some(reg) = self.get_reg() else { break };
                    self.instructions.push(Instruction::Load(reg));
                }
                /*
                        "creg" => {
                            let Some(reg) = self.get_reg() else { break };
                            self.instructions.push(Instruction::Creg(reg));
                        }
                        "clregs" => self.instructions.push(Instruction::Clregs),
                */
                "dumpreg" | "dr" => self.instructions.push(Instruction::DumpReg),

                // Vector
                "vreal" => {
                    let Some(reg) = self.get_reg() else { break };
                    self.instructions.push(Instruction::Vreal(reg));
                }
                /*
                        "vcplx" => {
                            let Some(reg) = self.get_reg() else { break };
                            self.instructions.push(Instruction::Vcplx(reg));
                        }
                */
                "vsave" => {
                    let Some(reg) = self.get_reg() else { break };
                    self.instructions.push(Instruction::Vsave(reg));
                }
                "vload" => {
                    let Some(reg) = self.get_reg() else { break };
                    self.instructions.push(Instruction::Vload(reg));
                }
                "cvec" => {
                    let Some(reg) = self.get_reg() else { break };
                    self.instructions.push(Instruction::Cvec(reg));
                }
                "clvecs" => self.instructions.push(Instruction::Clvecs),
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
                "dumpsr" | "dsr" => {
                    for p in &self.procedure_lut {
                        println!("Subroutine   {}", p.1 .1);
                    }
                }
                "[" => self
                    .loop_addr
                    .push(self.runner.get_proglen() + self.instructions.len()),
                "]" => self
                    .instructions
                    .push(Instruction::Jnz(self.loop_addr.pop().unwrap())),

                "realmode" | "rm" => {
                    if self.procedure_state == 0 {
                        self.complex_mode = false
                    } else {
                        eprintln!("Cannot change in procedure!");
                    }
                }
                "complexmode" | "cm" => {
                    if self.procedure_state == 0 {
                        self.complex_mode = true
                    } else {
                        eprintln!("Cannot change in procedure!");
                    }
                }
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
                            (
                                self.runner.get_proglen() + self.instructions.len(),
                                line.to_string(),
                            ),
                        );
                        self.procedure_state = 2;
                    } else if let Some((call_ptr, _description)) = self.procedure_lut.get(token) {
                        // token -> call subrutin
                        self.instructions.push(Instruction::Call(*call_ptr));
                    } else if token.as_bytes()[0].is_ascii_digit() || token.as_bytes()[0] == b'-' {
                        let Ok(number) = token.parse::<f64>() else {
                            eprintln!("Number error");
                            break;
                        };
                        self.instructions.push(Instruction::Literal(number));
                    } else {
                        eprintln!("Not a number, invalid command. Please type 'help'.");
                    }
                }
            } // match
        } // for token
        if self.procedure_state == 0 && !self.instructions.is_empty() {
            self.runner.run(&self.instructions);
            self.instructions.clear();
        }
    } // end fn parse
} // end Parse
