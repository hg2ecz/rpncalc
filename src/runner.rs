use crate::instructions::Instruction;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};

const MAX_STACK: usize = 1_000_000;

#[derive(Debug)]
pub struct RealRunner {
    fractionaldigit: usize,
    prog: Vec<Instruction>,
    pc: usize,
    ret_stack: Vec<usize>,
    stack: Vec<f64>,
    registers: [f64; 256],
    vectors: Vec<Vec<f64>>,
    verbose: bool,
    stopped: Arc<AtomicBool>,

    accumulator: f64,
}

impl RealRunner {
    pub fn new(verbose: bool) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let r = stopped.clone();

        ctrlc::set_handler(move || {
            r.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        let mut vectors = Vec::new();
        for _ in 0..256 {
            vectors.push(Vec::new())
        }
        RealRunner {
            fractionaldigit: 0,
            prog: vec![],
            pc: 0,
            stack: Vec::new(),
            ret_stack: Vec::new(),
            registers: [0.0; 256],
            vectors,
            verbose,
            stopped,

            accumulator: 0.0,
        }
    }

    pub fn get_proglen(&mut self) -> usize {
        self.prog.len()
    }

    // add procedure, without running. For procedures.
    pub fn add_instr(&mut self, add_instr: &[Instruction]) {
        for i in add_instr {
            self.prog.push(*i);
        }
        self.pc = self.prog.len();
    }

    // More stack element -> stack to workreg
    fn accu_last(&self, accu_in: &f64) -> Option<f64> {
        if self.stack.is_empty() {
            eprintln!("Stack is empty!");
            None
        } else {
            Some(*accu_in)
        }
    }

    fn accu_pop(&mut self, accu_in: &mut f64) -> Option<f64> {
        let accu = *accu_in;
        if let Some(a) = self.stack.pop() {
            *accu_in = a;
            Some(accu)
        } else {
            eprintln!("Stack is empty!");
            None
        }
    }
    fn accu_push(&mut self, accu_in: &mut f64, num: f64) -> bool {
        self.stack.push(*accu_in);
        *accu_in = num;
        if self.stack.len() >= MAX_STACK {
            eprintln!(
                "Stack is FULL ({} element)! Please clear it.",
                self.stack.len()
            );
            true // stack overflow error
        } else {
            false // no error
        }
    }

    pub fn run(&mut self, add_instr: &[Instruction]) {
        let mut err = false;
        for i in add_instr {
            self.prog.push(*i);
        }

        let mut spc = self.pc;
        let mut accu = self.accumulator;
        while spc < self.prog.len() {
            if self.verbose {
                println!("Debug: PC: {} Instr: {:?}", spc, self.prog[spc]);
            }
            match self.prog[spc] {
                Instruction::Literal(lit) => err |= self.accu_push(&mut accu, lit),
                Instruction::Call(addr) => {
                    self.ret_stack.push(spc);
                    spc = addr;
                    continue; // don't increment PC
                }
                Instruction::Ret => {
                    let Some(pc_ret) = self.ret_stack.pop() else {
                        eprintln!("RET: Return stack is empty!");
                        break;
                    };
                    spc = pc_ret;
                }
                Instruction::Jnz(addr) => {
                    let Some(a) = self.accu_pop(&mut accu) else {
                        eprintln!("JNZ: Stack is empty!");
                        break;
                    };
                    if self.stopped.load(Ordering::SeqCst) {
                        self.stopped.store(false, Ordering::SeqCst);
                        eprintln!("Ctrl-C ... stop");
                        break;
                    } else if a != 0.0 {
                        spc = addr;
                        continue;
                    }
                }

                // Stack operations
                Instruction::Dup => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    self.stack.push(a);
                    if self.stack.len() >= MAX_STACK {
                        eprintln!(
                            "Stack is FULL ({} element)! Please clear it.",
                            self.stack.len()
                        );
                        break;
                    }
                }
                Instruction::Drop => {
                    if self.accu_pop(&mut accu).is_none() {
                        eprintln!("Stack is empty!");
                        break;
                    }
                }
                Instruction::Over => {
                    let Some(&a) = self.stack.last() else {
                        eprintln!("Stack is empty!");
                        break;
                    };
                    err |= self.accu_push(&mut accu, a);
                }
                Instruction::Rot => {
                    if let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_pop(&mut accu))
                    {
                        //let Some(c) = self.accu_last(&accu) else { break };;
                        self.stack.push(b);
                        self.stack.push(a);
                        // self.accu_push(&mut accu, c);
                    } else {
                        eprintln!("Stack is empty!");
                        break;
                    }
                }
                Instruction::Swap => {
                    if let Some(a) = self.accu_pop(&mut accu) {
                        self.stack.push(a); // accu --> last
                                            // self.stack.push(b);
                    } else {
                        eprintln!("Stack is empty!");
                        break;
                    }
                }
                Instruction::Clear => {
                    self.stack.clear();
                }
                Instruction::DumpStack => {
                    if self.stack.is_empty() {
                        println!("Stack is empty!");
                    } else {
                        println!("Stack: {:?}, {:?}", &self.stack[1..], self.accu_last(&accu));
                    }
                }

                // Basic arithmetic
                Instruction::Add => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = b + a;
                }
                Instruction::Sub => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = b - a;
                }
                Instruction::Mul => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = b * a;
                }
                Instruction::Div => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = b / a;
                }
                Instruction::And => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = (b as u32 & a as u32) as f64;
                }
                Instruction::Or => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = (b as u32 | a as u32) as f64;
                }
                Instruction::Xor => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = (b as u32 ^ a as u32) as f64;
                }
                Instruction::Neg => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = (a as u32 ^ 0xffff_ffff) as f64;
                }
                Instruction::Shl => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = ((b as u32) << a as u32) as f64;
                }
                Instruction::Shr => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = ((b as u32) >> a as u32) as f64;
                }
                Instruction::Abs => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.abs();
                }
                Instruction::Floor => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.floor();
                }
                Instruction::Ceil => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.ceil();
                }
                Instruction::Round => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.round();
                }

                // Trigonometric function
                Instruction::CosR => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.cos();
                }
                Instruction::SinR => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.sin();
                }
                Instruction::TanR => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.tan();
                }
                Instruction::CosD => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    let a = a / 180. * std::f64::consts::PI;
                    accu = a.cos();
                }
                Instruction::SinD => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    let a = a / 180. * std::f64::consts::PI;
                    accu = a.sin();
                }
                Instruction::TanD => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    let a = a / 180. * std::f64::consts::PI;
                    accu = a.tan();
                }
                Instruction::AcosR => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.acos();
                }
                Instruction::AsinR => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.asin();
                }
                Instruction::AtanR => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.atan();
                }
                Instruction::AcosD => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.acos() * 180. / std::f64::consts::PI;
                }
                Instruction::AsinD => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.asin() * 180. / std::f64::consts::PI;
                }
                Instruction::AtanD => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.atan() * 180. / std::f64::consts::PI;
                }
                // Logarithm and exponential
                Instruction::Loge => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.ln();
                }
                Instruction::Log2 => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.log2();
                }
                Instruction::Log10 => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.log10();
                }
                Instruction::Logx => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = b.ln() / a.ln();
                }

                Instruction::Expe => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.exp();
                }
                Instruction::Exp2 => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = a.exp2();
                }
                Instruction::Exp10 => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = 10_f64.powf(a);
                }
                Instruction::Expx => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = b.powf(a);
                }
                Instruction::Gt => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = (b > a) as i32 as f64;
                }
                Instruction::Lt => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = (b < a) as i32 as f64;
                }
                Instruction::Ge => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = (b >= a) as i32 as f64;
                }
                Instruction::Le => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = (b <= a) as i32 as f64;
                }
                Instruction::Eq => {
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_last(&accu))
                    else {
                        break;
                    };
                    accu = (b == a) as i32 as f64;
                }

                // Registers
                Instruction::Save(regnum) => {
                    let Some(x) = self.accu_pop(&mut accu) else {
                        eprintln!("Stack is empty!");
                        break;
                    };
                    self.registers[regnum as usize] = x;
                }
                Instruction::Load(regnum) => {
                    err |= self.accu_push(&mut accu, self.registers[regnum as usize]);
                }
                Instruction::DumpReg => {
                    for (i, v) in self.registers.iter().enumerate() {
                        println!("Reg {i:3}: {v:?}");
                    }
                }

                // Vectors
                Instruction::Vreal(regnum) => {
                    // vector create complex - with LEN
                    let Some(a) = self.accu_pop(&mut accu) else {
                        break;
                    };
                    self.vectors[regnum as usize] = vec![0.0; a as usize];
                }

                Instruction::Vsave(regnum) => {
                    // vsaveX
                    let (Some(a), Some(b)) = (self.accu_pop(&mut accu), self.accu_pop(&mut accu))
                    else {
                        break;
                    };
                    self.vectors[regnum as usize][a as usize] = b;
                }
                Instruction::Vload(regnum) => {
                    // vloadX
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    accu = self.vectors[regnum as usize][a as usize];
                }
                Instruction::Cvec(regnum) => {
                    self.vectors[regnum as usize].clear();
                }
                Instruction::Clvecs => {
                    for r in &mut self.vectors.iter_mut() {
                        r.clear();
                    }
                    eprintln!("All self.vectors is cleared.");
                }
                Instruction::DumpVec => {
                    let mut ok = false;
                    for (i, v) in self.vectors.iter().enumerate() {
                        if !v.is_empty() {
                            println!("Vec {i:3}  len: {}", v.len());
                            ok = true;
                        }
                    }
                    if !ok {
                        println!("Not found any defined vectors. Use LEN VNUM vreal or LEN VNUM vcplx for create of real or complex vector.")
                    }
                }

                // Print and related
                Instruction::FractionalDigit => {
                    let Some(a) = self.accu_pop(&mut accu) else {
                        eprintln!("FractionalDigit");
                        break;
                    };
                    if a <= 17.0 {
                        self.fractionaldigit = a as usize;
                    }
                }
                Instruction::Print => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    if self.fractionaldigit > 0 {
                        println!("Result: {a:.*?}", self.fractionaldigit);
                    } else {
                        println!("Result: {a:?}");
                    }
                }

                Instruction::Quit => {
                    eprintln!("Exit from calculator. Bye.");
                    std::process::exit(0);
                }
            } // match
            spc += 1;
            if err {
                break;
            }
        } // while
          // if breaked, drop the remaining part of the program
        if spc < self.prog.len() {
            spc = self.prog.len();
        }
        self.accumulator = accu;
        self.pc = spc;
    } // fn run
} // Obj
