use crate::instructions::{Instruction, StackType};
use num_complex::Complex;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};

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
    precision: usize,
    prog: Vec<Instruction>,
    pc: usize,
    stack: Vec<StackType>,
    ret_stack: Vec<usize>,
    registers: [StackType; 256],
    vectors: Vec<VectorType>,
    verbose: bool,
    stopped: Arc<AtomicBool>,
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
            precision: 17,
            prog: vec![],
            pc: 0,
            stack: Vec::new(),
            ret_stack: Vec::new(),
            registers: [StackType::None; 256],
            vectors,
            verbose,
            stopped,
        }
    }

    pub fn get_proglen(&mut self) -> usize {
        self.prog.len()
    }

    // add procedure, without running
    pub fn add_instr(&mut self, add_instr: &[Instruction]) {
        for i in add_instr {
            self.prog.push(*i);
        }
        self.pc = self.prog.len();
    }

    // Internal func
    fn get_double(&mut self) -> Option<f64> {
        let Some(a) = self.stack.pop() else {eprintln!("Stack is empty!");return None};
        let StackType::Double(a) = a else {eprintln!("Get double: type error (Complex)");return None};
        Some(a)
    }

    // Internal func, return: Real:Real or Complex:Complex from any pair
    fn get_samenum(&mut self) -> Option<(StackType, StackType)> {
        let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) else {eprintln!("Stack empty!"); return None};
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
        self.stopped.store(false, Ordering::SeqCst);
        for i in add_instr {
            self.prog.push(*i);
        }
        while self.pc < self.prog.len() {
            if self.verbose {
                println!("Debug: PC: {} Instr: {:?}", self.pc, self.prog[self.pc]);
            }
            match self.prog[self.pc] {
                Instruction::Literal(lit) => self.stack.push(lit),
                Instruction::Call(addr) => {
                    self.ret_stack.push(self.pc);
                    self.pc = addr;
                    continue; // don't increment PC
                }
                Instruction::Ret => {
                    let Some(pc) = self.ret_stack.pop() else { eprintln!("Return stack is empty!"); break; };
                    self.pc = pc;
                }
                Instruction::Jnz(addr) => {
                    let Some(a) = self.stack.pop() else { eprintln!("Stack is empty!"); break; };
                    if self.stopped.load(Ordering::SeqCst) {
                        eprintln!("Ctrl-C ... stop");
                        break;
                    } else if a != StackType::Double(0.0) {
                        self.pc = addr;
                    }
                }

                // Stack operations
                Instruction::Dup => {
                    let Some(a) = self.stack.last() else { eprintln!("Stack is empty!"); break; };
                    self.stack.push(*a);
                }
                Instruction::Drop => {
                    if self.stack.pop().is_none() {
                        eprintln!("Stack is empty!");
                        break;
                    }
                }
                Instruction::Over => {
                    let Some(&a) = self.stack.get(self.stack.len() - 2) else {eprintln!("Stack is empty!");break};
                    self.stack.push(a);
                }
                Instruction::Rot => {
                    if let (Some(a), Some(b), Some(c)) =
                        (self.stack.pop(), self.stack.pop(), self.stack.pop())
                    {
                        self.stack.push(b);
                        self.stack.push(a);
                        self.stack.push(c);
                    } else {
                        eprintln!("Stack is empty!");
                        break;
                    }
                }
                Instruction::Swap => {
                    if let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) {
                        self.stack.push(a);
                        self.stack.push(b);
                    } else {
                        eprintln!("Stack is empty!");
                        break;
                    }
                }
                Instruction::Clear => {
                    self.stack.clear();
                }
                Instruction::DumpStack => {
                    println!("Stack: {:?}", self.stack);
                }

                // Basic arithmetic
                Instruction::Add => {
                    let Some((a, b)) = self.get_samenum() else {break};
                    if let (StackType::Double(a), StackType::Double(b)) = (a, b) {
                        self.stack.push(StackType::Double(b + a));
                    } else if let (StackType::Complex(a), StackType::Complex(b)) = (a, b) {
                        self.stack.push(StackType::Complex(b + a));
                    }
                }
                Instruction::Sub => {
                    let Some((a, b)) = self.get_samenum() else {break};
                    if let (StackType::Double(a), StackType::Double(b)) = (a, b) {
                        self.stack.push(StackType::Double(b - a));
                    } else if let (StackType::Complex(a), StackType::Complex(b)) = (a, b) {
                        self.stack.push(StackType::Complex(b - a));
                    }
                }
                Instruction::Mul => {
                    let Some((a, b)) = self.get_samenum() else {break};
                    if let (StackType::Double(a), StackType::Double(b)) = (a, b) {
                        self.stack.push(StackType::Double(b * a));
                    } else if let (StackType::Complex(a), StackType::Complex(b)) = (a, b) {
                        self.stack.push(StackType::Complex(b * a));
                    }
                }
                Instruction::Div => {
                    let Some((a, b)) = self.get_samenum() else {break};
                    if let (StackType::Double(a), StackType::Double(b)) = (a, b) {
                        self.stack.push(StackType::Double(b / a));
                    } else if let (StackType::Complex(a), StackType::Complex(b)) = (a, b) {
                        self.stack.push(StackType::Complex(b / a));
                    }
                }
                Instruction::And => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack
                        .push(StackType::Double((b as u32 & a as u32) as f64));
                }
                Instruction::Or => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack
                        .push(StackType::Double((b as u32 | a as u32) as f64));
                }
                Instruction::Xor => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack
                        .push(StackType::Double((b as u32 ^ a as u32) as f64));
                }
                Instruction::Neg => {
                    let Some(a) = self.get_double() else {break};
                    self.stack
                        .push(StackType::Double((a as u32 ^ 0xffff_ffff) as f64));
                }
                Instruction::Shl => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack
                        .push(StackType::Double(((b as u32) << a as u32) as f64));
                }
                Instruction::Shr => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack
                        .push(StackType::Double(((b as u32) >> a as u32) as f64));
                }
                Instruction::Abs => {
                    let Some(a) = self.stack.pop() else {break;};
                    if let StackType::Double(a) = a {
                        self.stack.push(StackType::Double(a.abs()));
                    } else if let StackType::Complex(a) = a {
                        self.stack.push(StackType::Double(a.norm()));
                    }
                }
                Instruction::Floor => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.floor()));
                }
                Instruction::Ceil => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.ceil()));
                }
                Instruction::Round => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.round()));
                }

                // Trigonometric function
                Instruction::CosR => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.cos()));
                }
                Instruction::SinR => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.sin()));
                }
                Instruction::TanR => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.tan()));
                }
                Instruction::CosD => {
                    let Some(a) = self.get_double() else {break};
                    let a = a / 180. * std::f64::consts::PI;
                    self.stack.push(StackType::Double(a.cos()));
                }
                Instruction::SinD => {
                    let Some(a) = self.get_double() else {break};
                    let a = a / 180. * std::f64::consts::PI;
                    self.stack.push(StackType::Double(a.sin()));
                }
                Instruction::TanD => {
                    let Some(a) = self.get_double() else {break};
                    let a = a / 180. * std::f64::consts::PI;
                    self.stack.push(StackType::Double(a.tan()));
                }
                Instruction::AcosR => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.acos()));
                }
                Instruction::AsinR => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.asin()));
                }
                Instruction::AtanR => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.atan()));
                }
                Instruction::AcosD => {
                    let Some(a) = self.get_double() else {break};
                    let a = a.acos() * 180. / std::f64::consts::PI;
                    self.stack.push(StackType::Double(a));
                }
                Instruction::AsinD => {
                    let Some(a) = self.get_double() else {break};
                    let a = a.asin() * 180. / std::f64::consts::PI;
                    self.stack.push(StackType::Double(a));
                }
                Instruction::AtanD => {
                    let Some(a) = self.get_double() else {break};
                    let a = a.atan() * 180. / std::f64::consts::PI;
                    self.stack.push(StackType::Double(a));
                }
                // Logarithm and exponential
                Instruction::Loge => {
                    let Some(a) = self.stack.pop() else {eprintln!("Stack is empty!");break};
                    match a {
                        StackType::Double(aa) => self.stack.push(StackType::Double(aa.ln())),
                        StackType::Complex(aa) => self.stack.push(StackType::Complex(aa.ln())),
                        _ => eprintln!("Loge type error."),
                    }
                }
                Instruction::Log2 => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.log2()));
                }
                Instruction::Log10 => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.log10()));
                }
                Instruction::Logx => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack.push(StackType::Double(b.ln() / a.ln()));
                }

                Instruction::Expe => {
                    let Some(a) = self.stack.pop() else {eprintln!("Stack is empty!");break};
                    match a {
                        StackType::Double(aa) => self.stack.push(StackType::Double(aa.exp())),
                        StackType::Complex(aa) => self.stack.push(StackType::Complex(aa.exp())),
                        _ => eprintln!("Exp type error."),
                    }
                }
                Instruction::Exp2 => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(a.exp2()));
                }
                Instruction::Exp10 => {
                    let Some(a) = self.get_double() else {break};
                    self.stack.push(StackType::Double(10_f64.powf(a)));
                }
                Instruction::Expx => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack.push(StackType::Double(b.powf(a)));
                }
                Instruction::Gt => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack.push(StackType::Double((b > a) as i32 as f64));
                }
                Instruction::Lt => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack.push(StackType::Double((b < a) as i32 as f64));
                }
                Instruction::Ge => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack.push(StackType::Double((b >= a) as i32 as f64));
                }
                Instruction::Le => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack.push(StackType::Double((b <= a) as i32 as f64));
                }
                Instruction::Eq => {
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.get_double() else {break};
                    self.stack.push(StackType::Double((b == a) as i32 as f64));
                }

                // Complex
                Instruction::Real => {
                    let Some(a) = self.stack.pop() else {eprintln!("Stack is empty!");break};
                    let StackType::Complex(a) = a else {eprintln!("This program compute trigonometric value only with Double, not Complex.");break};
                    self.stack.push(StackType::Double(a.re));
                }
                Instruction::Imag => {
                    let Some(a) = self.stack.pop() else {eprintln!("Stack is empty!");break};
                    let StackType::Complex(a) = a else {eprintln!("This program compute trigonometric value only with Double, not Complex.");break};
                    self.stack.push(StackType::Double(a.im));
                }
                Instruction::R2c => {
                    let (Some(a), Some(b)) = (self.stack.pop(), self.stack.pop()) else {eprintln!("Stack is empty!");break};
                    let (StackType::Double(a), StackType::Double(b)) = (a, b) else {eprintln!("Numbers are not real!"); break};
                    self.stack.push(StackType::Complex(Complex::new(b, a)));
                }

                // Registers
                Instruction::Save(regnum) => {
                    let Some(x) = self.stack.pop() else { eprintln!("Stack is empty!"); break; };
                    self.registers[regnum as usize] = x;
                }
                Instruction::Load(regnum) => {
                    self.stack.push(self.registers[regnum as usize]);
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
                    let Some(a) = self.get_double() else {break};
                    self.vectors[regnum as usize].data_type = Type::Double;
                    self.vectors[regnum as usize].vector = vec![0.0; a as usize];
                }
                Instruction::Vcplx(regnum) => {
                    // vector create complex - with LEN
                    let Some(a) = self.get_double() else {break};
                    self.vectors[regnum as usize].data_type = Type::Complex;
                    self.vectors[regnum as usize].vector = vec![0.0; 2 * a as usize];
                }
                Instruction::Vsave(regnum) => {
                    // vsaveX
                    let Some(a) = self.get_double() else {break};
                    let Some(b) = self.stack.pop() else {eprintln!("Stack empty"); break};
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
                    let Some(a) = self.get_double() else {break};
                    if self.vectors[regnum as usize].data_type == Type::Double {
                        self.stack.push(StackType::Double(
                            self.vectors[regnum as usize].vector[a as usize],
                        ));
                    } else {
                        // Complex
                        self.stack.push(StackType::Complex(Complex::new(
                            self.vectors[regnum as usize].vector[2 * a as usize],
                            self.vectors[regnum as usize].vector[2 * a as usize + 1],
                        )));
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
                Instruction::Precision => {
                    let Some(StackType::Double(a)) = self.stack.pop() else {eprintln!("Precision error"); break};
                    if a <= 17.0 {
                        self.precision = a as usize;
                    }
                }
                Instruction::GetPrecision => println!("Precision: {}", self.precision),

                Instruction::Print => {
                    let Some(a) = self.stack.last() else {eprintln!("Stack is empty.");break};
                    println!("{a:.*?}", self.precision);
                }

                Instruction::Quit => {
                    eprintln!("Exit from calculator. Bye.");
                    std::process::exit(0);
                }
            } // match
            self.pc += 1;
        } // while
          // if breaked, drop the remaining part of the program
        if self.pc < self.prog.len() {
            self.pc = self.prog.len();
        }
    } // fn run
} // Obj
