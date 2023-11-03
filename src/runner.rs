use crate::instructions::Instruction;
use num_complex::Complex;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};

const MAX_STACK: usize = 1_000_000;

#[derive(Debug)]
pub struct Runner {
    fractionaldigit: usize,
    prog: Vec<Instruction>,
    pc: usize,
    ret_stack: Vec<usize>,
    stack: Vec<f64>,
    registers: [f64; 256],
    vectors: Vec<Vec<f64>>,

    cplx_stack: Vec<Complex<f64>>,
    cplx_registers: [Complex<f64>; 256],
    cplx_vectors: Vec<Vec<Complex<f64>>>,

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
        let mut cplx_vectors = Vec::new();
        for _ in 0..256 {
            vectors.push(Vec::new());
            cplx_vectors.push(Vec::new());
        }
        Runner {
            fractionaldigit: 0,
            prog: vec![],
            pc: 0,
            ret_stack: Vec::new(),
            stack: Vec::new(),
            registers: [0.0; 256],
            vectors,

            cplx_stack: Vec::new(),
            cplx_registers: [Complex::new(0.0, 0.0); 256],
            cplx_vectors,

            verbose,
            stopped,
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

    fn accu_pop(&mut self) -> Option<f64> {
        if let Some(a) = self.stack.pop() {
            Some(a)
        } else {
            eprintln!("Stack is empty!");
            None
        }
    }
    fn accu_push(&mut self, num: f64) -> bool {
        if self.stack.len() >= MAX_STACK {
            eprintln!(
                "Stack is FULL ({} element)! Please clear it.",
                self.stack.len()
            );
            true // stack overflow error
        } else {
            self.stack.push(num);
            false // no error
        }
    }

    fn cplx_accu_pop(&mut self) -> Option<Complex<f64>> {
        if let Some(a) = self.cplx_stack.pop() {
            Some(a)
        } else {
            eprintln!("Complex Stack is empty!");
            None
        }
    }
    fn cplx_accu_push(&mut self, num: Complex<f64>) -> bool {
        if self.cplx_stack.len() >= MAX_STACK {
            eprintln!(
                "Complex Stack is FULL ({} element)! Please clear it.",
                self.cplx_stack.len()
            );
            true // stack overflow error
        } else {
            self.cplx_stack.push(num);
            false // no error
        }
    }

    pub fn run(&mut self, add_instr: &[Instruction]) {
        let mut err = false;
        for i in add_instr {
            self.prog.push(*i);
        }

        while !err && self.pc < self.prog.len() {
            if self.verbose {
                println!("Debug: PC: {} Instr: {:?}", self.pc, self.prog[self.pc]);
            }
            match self.prog[self.pc] {
                Instruction::Literal(lit) => err = self.accu_push(lit),
                Instruction::Call(addr) => {
                    self.ret_stack.push(self.pc);
                    self.pc = addr;
                    continue; // don't increment PC
                }
                Instruction::Ret => {
                    if let Some(pc_ret) = self.ret_stack.pop() {
                        self.pc = pc_ret;
                    } else {
                        eprintln!("RET: Return stack is empty!");
                        err = true;
                    }
                }
                Instruction::Jnz(addr) => {
                    if let Some(a) = self.accu_pop() {
                        if self.stopped.load(Ordering::SeqCst) {
                            self.stopped.store(false, Ordering::SeqCst);
                            eprintln!("Ctrl-C ... stop");
                            break; // exit
                        }
                        if a != 0.0 {
                            self.pc = addr;
                            continue;
                        }
                    } else {
                        err = true;
                    }
                }

                // Stack operations
                Instruction::Dup => {
                    if let Some(&a) = self.stack.last() {
                        err = self.accu_push(a); // check
                    } else {
                        eprintln!("Stack is empty!");
                        err = true;
                    }
                }
                Instruction::Drop => {
                    err = self.accu_pop().is_none();
                }
                Instruction::Over => {
                    if let Some(&a) = self.stack.get(self.stack.len() - 2) {
                        err = self.accu_push(a);
                    } else {
                        eprintln!("Stack is empty!");
                        err = true;
                    }
                }
                Instruction::Rot => {
                    if let (Some(a), Some(b), Some(c)) =
                        (self.accu_pop(), self.accu_pop(), self.accu_pop())
                    {
                        self.stack.push(b);
                        self.stack.push(a);
                        self.stack.push(c);
                    } else {
                        err = true;
                    }
                }
                Instruction::Swap => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push(a);
                        self.stack.push(b);
                    } else {
                        err = true;
                    }
                }
                Instruction::Clear => {
                    self.stack.clear();
                }
                Instruction::DumpStack => {
                    println!("Stack: {:?}", &self.stack);
                }

                // Basic arithmetic
                Instruction::Add => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push(b + a);
                    } else {
                        err = true;
                    }
                }
                Instruction::Sub => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push(b - a);
                    } else {
                        err = true;
                    }
                }
                Instruction::Mul => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push(b * a);
                    } else {
                        err = true;
                    }
                }
                Instruction::Div => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push(b / a);
                    } else {
                        err = true;
                    }
                }
                Instruction::And => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push((b as u32 & a as u32) as f64);
                    } else {
                        err = true;
                    }
                }
                Instruction::Or => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push((b as u32 | a as u32) as f64);
                    } else {
                        err = true;
                    }
                }
                Instruction::Xor => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push((b as u32 ^ a as u32) as f64);
                    } else {
                        err = true;
                    }
                }
                Instruction::Neg => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push((a as u32 ^ 0xffff_ffff) as f64);
                    } else {
                        err = true;
                    }
                }
                Instruction::Shl => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push(((b as u32) << a as u32) as f64);
                    } else {
                        err = true;
                    }
                }
                Instruction::Shr => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push(((b as u32) >> a as u32) as f64);
                    } else {
                        err = true;
                    };
                }
                Instruction::Abs => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.abs());
                    } else {
                        err = true;
                    }
                }
                Instruction::Floor => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.floor());
                    } else {
                        err = true;
                    }
                }
                Instruction::Ceil => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.ceil());
                    } else {
                        err = true;
                    }
                }
                Instruction::Round => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.round());
                    } else {
                        err = true;
                    }
                }

                // Trigonometric function
                Instruction::CosR => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.cos());
                    } else {
                        err = true;
                    }
                }
                Instruction::SinR => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.sin());
                    } else {
                        err = true;
                    }
                }
                Instruction::TanR => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.tan());
                    } else {
                        err = true;
                    }
                }
                Instruction::CosD => {
                    if let Some(a) = self.accu_pop() {
                        let a = a / 180. * std::f64::consts::PI;
                        self.stack.push(a.cos());
                    } else {
                        err = true;
                    }
                }
                Instruction::SinD => {
                    if let Some(a) = self.accu_pop() {
                        let a = a / 180. * std::f64::consts::PI;
                        self.stack.push(a.sin());
                    } else {
                        err = true;
                    }
                }
                Instruction::TanD => {
                    if let Some(a) = self.accu_pop() {
                        let a = a / 180. * std::f64::consts::PI;
                        self.stack.push(a.tan());
                    } else {
                        err = true;
                    }
                }
                Instruction::AcosR => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.acos());
                    } else {
                        err = true;
                    }
                }
                Instruction::AsinR => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.asin());
                    } else {
                        err = true;
                    }
                }
                Instruction::AtanR => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.atan());
                    } else {
                        err = true;
                    }
                }
                Instruction::AcosD => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.acos() * 180. / std::f64::consts::PI);
                    } else {
                        err = true;
                    }
                }
                Instruction::AsinD => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.asin() * 180. / std::f64::consts::PI);
                    } else {
                        err = true;
                    }
                }
                Instruction::AtanD => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.atan() * 180. / std::f64::consts::PI);
                    } else {
                        err = true;
                    }
                }
                // Logarithm and exponential
                Instruction::Loge => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.ln());
                    } else {
                        err = true;
                    }
                }
                Instruction::Log2 => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.log2());
                    } else {
                        err = true;
                    }
                }
                Instruction::Log10 => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.log10());
                    } else {
                        err = true;
                    }
                }
                Instruction::Logx => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push(b.ln() / a.ln());
                    } else {
                        err = true;
                    };
                }

                Instruction::Expe => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.exp());
                    } else {
                        err = true;
                    }
                }
                Instruction::Exp2 => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(a.exp2());
                    } else {
                        err = true;
                    }
                }
                Instruction::Exp10 => {
                    if let Some(a) = self.accu_pop() {
                        self.stack.push(10_f64.powf(a));
                    } else {
                        err = true;
                    }
                }
                Instruction::Expx => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push(b.powf(a));
                    } else {
                        err = true;
                    }
                }
                Instruction::Gt => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push((b > a) as i32 as f64);
                    } else {
                        err = true;
                    }
                }
                Instruction::Lt => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push((b < a) as i32 as f64);
                    } else {
                        err = true;
                    }
                }
                Instruction::Ge => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push((b >= a) as i32 as f64);
                    } else {
                        err = true;
                    }
                }
                Instruction::Le => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push((b <= a) as i32 as f64);
                    } else {
                        err = true;
                    }
                }
                Instruction::Eq => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.stack.push((b == a) as i32 as f64);
                    } else {
                        err = true;
                    }
                }

                // Registers
                Instruction::Save(regnum) => {
                    if let Some(a) = self.accu_pop() {
                        self.registers[regnum as usize] = a;
                    } else {
                        eprintln!("Stack is empty!");
                        err = true;
                    }
                }
                Instruction::Load(regnum) => {
                    err = self.accu_push(self.registers[regnum as usize]);
                }
                Instruction::DumpReg => {
                    for (i, v) in self.registers.iter().enumerate() {
                        println!("Reg {i:3}: {v:?}");
                    }
                }

                // Vectors
                Instruction::Vcreate(regnum) => {
                    // vector create complex - with LEN
                    if let Some(a) = self.accu_pop() {
                        self.vectors[regnum as usize] = vec![0.0; a as usize];
                    } else {
                        err = true;
                    }
                }

                Instruction::Vsave(regnum) => {
                    // vsaveX
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.vectors[regnum as usize][a as usize] = b;
                    } else {
                        err = true;
                    }
                }
                Instruction::Vload(regnum) => {
                    // vloadX
                    if let Some(a) = self.accu_pop() {
                        err = self.accu_push(self.vectors[regnum as usize][a as usize]);
                    } else {
                        err = true;
                    };
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
                    if let Some(a) = self.accu_pop() {
                        if a <= 17.0 {
                            self.fractionaldigit = a as usize;
                        }
                    } else {
                        eprintln!("FractionalDigit");
                        err = true;
                    }
                }
                Instruction::Print => {
                    if let Some(a) = self.stack.last() {
                        if self.fractionaldigit > 0 {
                            println!("Result: {a:.*?}", self.fractionaldigit);
                        } else {
                            println!("Result: {a:?}");
                        }
                    } else {
                        eprintln!("Error: accu is empty!");
                        err = true;
                    }
                }

                // === Complex ===
                Instruction::CplxReal => {
                    if let Some(a) = self.cplx_accu_pop() {
                        self.stack.push(a.re);
                    } else {
                        err = true;
                    }
                }
                Instruction::CplxImag => {
                    if let Some(a) = self.cplx_accu_pop() {
                        self.stack.push(a.im);
                    } else {
                        err = true;
                    }
                }

                Instruction::CplxR2c => {
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.accu_pop()) {
                        self.cplx_stack.push(Complex::new(b, a));
                    } else {
                        err = true;
                    }
                }
                Instruction::CplxC2r => {
                    if let Some(a) = self.cplx_accu_pop() {
                        self.stack.push(a.re);
                        self.stack.push(a.im);
                    } else {
                        err = true;
                    }
                }

                // Complex stack operation
                Instruction::CplxDup => {
                    if let Some(&a) = self.cplx_stack.last() {
                        err = self.cplx_accu_push(a); // check
                    } else {
                        eprintln!("Stack is empty!");
                        err = true;
                    }
                }
                Instruction::CplxDrop => {
                    err = self.cplx_accu_pop().is_none();
                }
                Instruction::CplxOver => {
                    if let Some(&a) = self.cplx_stack.get(self.cplx_stack.len() - 2) {
                        err = self.cplx_accu_push(a);
                    } else {
                        eprintln!("Stack is empty!");
                        err = true;
                    }
                }
                Instruction::CplxRot => {
                    if let (Some(a), Some(b), Some(c)) = (
                        self.cplx_accu_pop(),
                        self.cplx_accu_pop(),
                        self.cplx_accu_pop(),
                    ) {
                        self.cplx_stack.push(b);
                        self.cplx_stack.push(a);
                        self.cplx_stack.push(c);
                    } else {
                        err = true;
                    }
                }
                Instruction::CplxSwap => {
                    if let (Some(a), Some(b)) = (self.cplx_accu_pop(), self.cplx_accu_pop()) {
                        self.cplx_stack.push(a);
                        self.cplx_stack.push(b);
                    } else {
                        err = true;
                    }
                }
                Instruction::CplxClear => {
                    self.cplx_stack.clear();
                }
                Instruction::CplxDumpStack => {
                    println!("Stack: {:?}", &self.cplx_stack);
                }

                // Complex arithmetic
                Instruction::CplxAdd => {
                    if let (Some(a), Some(b)) = (self.cplx_accu_pop(), self.cplx_accu_pop()) {
                        self.cplx_stack.push(b + a);
                    } else {
                        err = true;
                    }
                }
                Instruction::CplxSub => {
                    if let (Some(a), Some(b)) = (self.cplx_accu_pop(), self.cplx_accu_pop()) {
                        self.cplx_stack.push(b - a);
                    } else {
                        err = true;
                    }
                }
                Instruction::CplxMul => {
                    if let (Some(a), Some(b)) = (self.cplx_accu_pop(), self.cplx_accu_pop()) {
                        self.cplx_stack.push(b * a);
                    } else {
                        err = true;
                    }
                }
                Instruction::CplxDiv => {
                    if let (Some(a), Some(b)) = (self.cplx_accu_pop(), self.cplx_accu_pop()) {
                        self.cplx_stack.push(b / a);
                    } else {
                        err = true;
                    }
                }

                // complex -> f64
                Instruction::CplxAbs => {
                    if let Some(a) = self.cplx_accu_pop() {
                        self.stack.push(a.norm());
                    } else {
                        err = true;
                    }
                }

                // ComplexRegisters
                Instruction::CplxSave(regnum) => {
                    if let Some(a) = self.cplx_accu_pop() {
                        self.cplx_registers[regnum as usize] = a;
                    } else {
                        eprintln!("Complex Stack is empty!");
                        err = true;
                    }
                }
                Instruction::CplxLoad(regnum) => {
                    err = self.cplx_accu_push(self.cplx_registers[regnum as usize]);
                }
                Instruction::CplxDumpReg => {
                    for (i, v) in self.cplx_registers.iter().enumerate() {
                        println!("Reg {i:3}: {v:?}");
                    }
                }

                // Complex Vectors
                // size: from f64 vector
                Instruction::CplxVcreate(regnum) => {
                    // vector create complex - with LEN
                    if let Some(a) = self.accu_pop() {
                        self.cplx_vectors[regnum as usize] =
                            vec![Complex::new(0.0, 0.0); a as usize];
                    } else {
                        err = true;
                    }
                }

                // idx: from f64 vector
                Instruction::CplxVsave(regnum) => {
                    // vsaveX
                    if let (Some(a), Some(b)) = (self.accu_pop(), self.cplx_accu_pop()) {
                        self.cplx_vectors[regnum as usize][a as usize] = b;
                    } else {
                        err = true;
                    }
                }
                // idx: from f64 vector
                Instruction::CplxVload(regnum) => {
                    // vloadX
                    if let Some(a) = self.accu_pop() {
                        err = self.cplx_accu_push(self.cplx_vectors[regnum as usize][a as usize]);
                    } else {
                        err = true;
                    };
                }
                Instruction::CplxCvec(regnum) => {
                    self.cplx_vectors[regnum as usize].clear();
                }
                Instruction::CplxClvecs => {
                    for r in &mut self.cplx_vectors.iter_mut() {
                        r.clear();
                    }
                    eprintln!("All self.cplx_vectors is cleared.");
                }
                Instruction::CplxDumpVec => {
                    let mut ok = false;
                    for (i, v) in self.cplx_vectors.iter().enumerate() {
                        if !v.is_empty() {
                            println!("Vec {i:3}  len: {}", v.len());
                            ok = true;
                        }
                    }
                    if !ok {
                        println!("Not found any defined vectors. Use LEN VNUM vreal or LEN VNUM vcplx for create of real or complex vector.")
                    }
                }

                Instruction::CplxPrint => {
                    if let Some(a) = self.cplx_stack.last() {
                        if self.fractionaldigit > 0 {
                            println!("Result: {a:.*?}", self.fractionaldigit);
                        } else {
                            println!("Result: {a:?}");
                        }
                    } else {
                        eprintln!("Error: Complex accu is empty!");
                        err = true;
                    }
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
