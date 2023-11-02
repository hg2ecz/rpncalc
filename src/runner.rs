use crate::instructions::{Instruction, StackType};
use num_complex::Complex;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};

const MAX_STACK: usize = 1_000_000;

#[derive(Debug, PartialEq)]
enum Type {
    Double,
    Complex,
}

#[derive(Debug)]
struct VectorType {
    data_type: Type,
    vector: Vec<f64>,
}

#[derive(Debug)]
pub struct Runner {
    fractionaldigit: usize,
    prog: Vec<Instruction>,
    pc: usize,
    stack: Vec<StackType>,
    ret_stack: Vec<usize>,
    registers: [StackType; 256],
    vectors: Vec<VectorType>,
    verbose: bool,
    stopped: Arc<AtomicBool>,

    accumulator: StackType,
}

impl Runner {
    pub fn new(verbose: bool) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let r = stopped.clone();

        ctrlc::set_handler(move || {
            r.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        let mut vectors = Vec::new();
        for _ in 0..256 {
            vectors.push(VectorType {
                data_type: Type::Double,
                vector: Vec::new(),
            })
        }
        Runner {
            fractionaldigit: 0,
            prog: vec![],
            pc: 0,
            stack: Vec::new(),
            ret_stack: Vec::new(),
            registers: [StackType::None; 256],
            vectors,
            verbose,
            stopped,

            accumulator: StackType::None,
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
    fn accu_last(&self, accu_in: &StackType) -> Option<StackType> {
        if self.stack.is_empty() {
            eprintln!("Stack is empty!");
            None
        } else {
            Some(*accu_in)
        }
    }

    fn accu_pop(&mut self, accu_in: &mut StackType) -> Option<StackType> {
        let accu = *accu_in;
        if let Some(a) = self.stack.pop() {
            *accu_in = a;
            Some(accu)
        } else {
            eprintln!("Stack is empty!");
            None
        }
    }
    fn accu_push(&mut self, accu_in: &mut StackType, num: StackType) -> bool {
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

    fn double_pop(&mut self, accu: &mut StackType) -> Option<f64> {
        let Some(a) = self.accu_pop(accu) else {
            return None;
        };
        let StackType::Double(a) = a else {
            eprintln!("Get double: type error (Complex)");
            return None;
        };
        Some(a)
    }

    fn double_last(&self, accu: &StackType) -> Option<f64> {
        if self.stack.is_empty() {
            eprintln!("Stack is empty!");
            return None;
        }
        let StackType::Double(a) = accu else {
            eprintln!("Get double: type error (Complex)");
            return None;
        };
        Some(*a)
    }

    // Internal func, return: Real:Real or Complex:Complex from any pair
    fn get_samenum(&mut self, accu: &mut StackType) -> Option<(StackType, StackType)> {
        let Some(a) = self.accu_pop(accu) else {
            return None;
        };
        if self.stack.is_empty() {
            eprintln!("Stack is empty!");
            return None;
        }
        let b = *accu;

        if let (StackType::Double(da), StackType::Double(db)) = (a, b) {
            Some((StackType::Double(da), StackType::Double(db)))
        } else if let (StackType::Complex(da), StackType::Complex(db)) = (a, b) {
            Some((StackType::Complex(da), StackType::Complex(db)))
        } else if let (StackType::Double(da), StackType::Complex(db)) = (a, b) {
            Some((
                StackType::Complex(Complex::new(da, 0.0)),
                StackType::Complex(db),
            ))
        } else if let (StackType::Complex(da), StackType::Double(db)) = (a, b) {
            Some((
                StackType::Complex(da),
                StackType::Complex(Complex::new(db, 0.0)),
            ))
        } else {
            eprintln!("Not a number!");
            None
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
                        eprintln!("Return stack is empty!");
                        break;
                    };
                    spc = pc_ret;
                }
                Instruction::Jnz(addr) => {
                    let Some(a) = self.accu_pop(&mut accu) else {
                        eprintln!("Stack is empty!");
                        break;
                    };
                    if self.stopped.load(Ordering::SeqCst) {
                        self.stopped.store(false, Ordering::SeqCst);
                        eprintln!("Ctrl-C ... stop");
                        break;
                    } else if a != StackType::Double(0.0) {
                        spc = addr;
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
                    let Some(a) = self.accu_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.accu_pop(&mut accu) else {
                        break;
                    };
                    //let Some(c) = self.accu_last(&accu) else { break };;
                    self.stack.push(b);
                    self.stack.push(a);
                    // self.accu_push(&mut accu, c);
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
                    let Some((a, b)) = self.get_samenum(&mut accu) else {
                        break;
                    };
                    if let (StackType::Double(a), StackType::Double(b)) = (a, b) {
                        accu = StackType::Double(b + a);
                    } else if let (StackType::Complex(a), StackType::Complex(b)) = (a, b) {
                        accu = StackType::Complex(b + a);
                    }
                }
                Instruction::Sub => {
                    let Some((a, b)) = self.get_samenum(&mut accu) else {
                        break;
                    };
                    if let (StackType::Double(a), StackType::Double(b)) = (a, b) {
                        accu = StackType::Double(b - a);
                    } else if let (StackType::Complex(a), StackType::Complex(b)) = (a, b) {
                        accu = StackType::Complex(b - a);
                    }
                }
                Instruction::Mul => {
                    let Some((a, b)) = self.get_samenum(&mut accu) else {
                        break;
                    };
                    if let (StackType::Double(a), StackType::Double(b)) = (a, b) {
                        accu = StackType::Double(b * a);
                    } else if let (StackType::Complex(a), StackType::Complex(b)) = (a, b) {
                        accu = StackType::Complex(b * a);
                    }
                }
                Instruction::Div => {
                    let Some((a, b)) = self.get_samenum(&mut accu) else {
                        break;
                    };
                    if let (StackType::Double(a), StackType::Double(b)) = (a, b) {
                        accu = StackType::Double(b / a);
                    } else if let (StackType::Complex(a), StackType::Complex(b)) = (a, b) {
                        accu = StackType::Complex(b / a);
                    }
                }
                Instruction::And => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double((b as u32 & a as u32) as f64);
                }
                Instruction::Or => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double((b as u32 | a as u32) as f64);
                }
                Instruction::Xor => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double((b as u32 ^ a as u32) as f64);
                }
                Instruction::Neg => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double((a as u32 ^ 0xffff_ffff) as f64);
                }
                Instruction::Shl => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(((b as u32) << a as u32) as f64);
                }
                Instruction::Shr => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(((b as u32) >> a as u32) as f64);
                }
                Instruction::Abs => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    if let StackType::Double(a) = a {
                        accu = StackType::Double(a.abs());
                    } else if let StackType::Complex(a) = a {
                        accu = StackType::Double(a.norm());
                    }
                }
                Instruction::Floor => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.floor());
                }
                Instruction::Ceil => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.ceil());
                }
                Instruction::Round => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.round());
                }

                // Trigonometric function
                Instruction::CosR => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.cos());
                }
                Instruction::SinR => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.sin());
                }
                Instruction::TanR => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.tan());
                }
                Instruction::CosD => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    let a = a / 180. * std::f64::consts::PI;
                    accu = StackType::Double(a.cos());
                }
                Instruction::SinD => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    let a = a / 180. * std::f64::consts::PI;
                    accu = StackType::Double(a.sin());
                }
                Instruction::TanD => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    let a = a / 180. * std::f64::consts::PI;
                    accu = StackType::Double(a.tan());
                }
                Instruction::AcosR => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.acos());
                }
                Instruction::AsinR => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.asin());
                }
                Instruction::AtanR => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.atan());
                }
                Instruction::AcosD => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    let a = a.acos() * 180. / std::f64::consts::PI;
                    accu = StackType::Double(a);
                }
                Instruction::AsinD => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    let a = a.asin() * 180. / std::f64::consts::PI;
                    accu = StackType::Double(a);
                }
                Instruction::AtanD => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    let a = a.atan() * 180. / std::f64::consts::PI;
                    accu = StackType::Double(a);
                }
                // Logarithm and exponential
                Instruction::Loge => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    match a {
                        StackType::Double(aa) => {
                            err |= self.accu_push(&mut accu, StackType::Double(aa.ln()))
                        }
                        StackType::Complex(aa) => accu = StackType::Complex(aa.ln()),
                        _ => eprintln!("Loge type error."),
                    }
                }
                Instruction::Log2 => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.log2());
                }
                Instruction::Log10 => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.log10());
                }
                Instruction::Logx => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(b.ln() / a.ln());
                }

                Instruction::Expe => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    match a {
                        StackType::Double(aa) => accu = StackType::Double(aa.exp()),
                        StackType::Complex(aa) => accu = StackType::Complex(aa.exp()),
                        _ => eprintln!("Exp type error."),
                    }
                }
                Instruction::Exp2 => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(a.exp2());
                }
                Instruction::Exp10 => {
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(10_f64.powf(a));
                }
                Instruction::Expx => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double(b.powf(a));
                }
                Instruction::Gt => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double((b > a) as i32 as f64);
                }
                Instruction::Lt => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double((b < a) as i32 as f64);
                }
                Instruction::Ge => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double((b >= a) as i32 as f64);
                }
                Instruction::Le => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double((b <= a) as i32 as f64);
                }
                Instruction::Eq => {
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.double_last(&accu) else {
                        break;
                    };
                    accu = StackType::Double((b == a) as i32 as f64);
                }

                // Complex
                Instruction::Real => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    let StackType::Complex(a) = a else {
                        eprintln!("This program compute trigonometric value only with Double, not Complex.");
                        break;
                    };
                    accu = StackType::Double(a.re);
                }
                Instruction::Imag => {
                    let Some(a) = self.accu_last(&accu) else {
                        break;
                    };
                    let StackType::Complex(a) = a else {
                        eprintln!("This program compute trigonometric value only with Double, not Complex.");
                        break;
                    };
                    accu = StackType::Double(a.im);
                }
                Instruction::R2c => {
                    let Some(a) = self.accu_pop(&mut accu) else {
                        eprintln!("Stack is empty!");
                        break;
                    };
                    let Some(b) = self.accu_last(&accu) else {
                        break;
                    };
                    let (StackType::Double(a), StackType::Double(b)) = (a, b) else {
                        eprintln!("Numbers are not real!");
                        break;
                    };
                    accu = StackType::Complex(Complex::new(b, a));
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
                Instruction::Creg(regnum) => {
                    self.registers[regnum as usize] = StackType::None;
                }
                Instruction::Clregs => {
                    for r in &mut self.registers.iter_mut() {
                        *r = StackType::None;
                    }
                    eprintln!("All self.registers is cleared.");
                }
                Instruction::DumpReg => {
                    let mut ok = false;
                    for (i, v) in self.registers.iter().enumerate() {
                        if *v != StackType::None {
                            println!("Reg {i:3}: {v:?}");
                            ok = true;
                        }
                    }
                    if !ok {
                        println!("Not found any defined registers. Use RNUM save for save.")
                    }
                }

                // Vectors
                Instruction::Vreal(regnum) => {
                    // vector create complex - with LEN
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    self.vectors[regnum as usize].data_type = Type::Double;
                    self.vectors[regnum as usize].vector = vec![0.0; a as usize];
                }
                Instruction::Vcplx(regnum) => {
                    // vector create complex - with LEN
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    self.vectors[regnum as usize].data_type = Type::Complex;
                    self.vectors[regnum as usize].vector = vec![0.0; 2 * a as usize];
                }
                Instruction::Vsave(regnum) => {
                    // vsaveX
                    let Some(a) = self.double_pop(&mut accu) else {
                        break;
                    };
                    let Some(b) = self.accu_pop(&mut accu) else {
                        eprintln!("Stack empty");
                        break;
                    };
                    match b {
                        StackType::Double(bb) => {
                            if self.vectors[regnum as usize].data_type != Type::Double {
                                eprintln!("Type error: vector is a real vector.");
                                break;
                            }
                            self.vectors[regnum as usize].vector[a as usize] = bb
                        }
                        StackType::Complex(bb) => {
                            if self.vectors[regnum as usize].data_type != Type::Complex {
                                eprintln!("Type error: vector is a complex vector.");
                                break;
                            }
                            self.vectors[regnum as usize].vector[2 * a as usize] = bb.re;
                            self.vectors[regnum as usize].vector[2 * a as usize + 1] = bb.im;
                        }
                        StackType::None => (),
                    }
                }
                Instruction::Vload(regnum) => {
                    // vloadX
                    let Some(a) = self.double_last(&accu) else {
                        break;
                    };
                    if self.vectors[regnum as usize].data_type == Type::Double {
                        accu = StackType::Double(self.vectors[regnum as usize].vector[a as usize]);
                    } else {
                        // Complex
                        accu = StackType::Complex(Complex::new(
                            self.vectors[regnum as usize].vector[2 * a as usize],
                            self.vectors[regnum as usize].vector[2 * a as usize + 1],
                        ));
                    }
                }
                Instruction::Cvec(regnum) => {
                    self.vectors[regnum as usize].vector.clear();
                }
                Instruction::Clvecs => {
                    for r in &mut self.vectors.iter_mut() {
                        r.vector.clear();
                    }
                    eprintln!("All self.vectors is cleared.");
                }
                Instruction::DumpVec => {
                    let mut ok = false;
                    for (i, v) in self.vectors.iter().enumerate() {
                        if !v.vector.is_empty() {
                            let mut vlen = v.vector.len();
                            if v.data_type == Type::Complex {
                                vlen /= 2;
                            }
                            println!("Vec {i:3}: {:?}, len: {vlen}", v.data_type);
                            ok = true;
                        }
                    }
                    if !ok {
                        println!("Not found any defined vectors. Use LEN VNUM vreal or LEN VNUM vcplx for create of real or complex vector.")
                    }
                }

                // Print and related
                Instruction::FractionalDigit => {
                    let Some(StackType::Double(a)) = self.accu_pop(&mut accu) else {
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
                    match a {
                        StackType::Double(res) => {
                            if self.fractionaldigit > 0 {
                                println!("Result: {res:.*?}", self.fractionaldigit);
                            } else {
                                println!("Result: {res:?}");
                            }
                        }
                        StackType::Complex(res) => {
                            if self.fractionaldigit > 0 {
                                println!("Result: {res:.*?}", self.fractionaldigit);
                            } else {
                                println!("Result: {res:?}");
                            }
                        }
                        _ => (),
                    };
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
