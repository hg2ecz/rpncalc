use num_complex::Complex;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StackType {
    Double(f64),
    Complex(Complex<f64>),
    //Str(&str),
    None,
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    Literal(StackType),
    Call(usize), // ":"
    Ret,         // ";"
    // "[", storing PC for JNZ
    Jnz(usize), // "]", jump back
    Save(u8),   // "save" + Reg
    Load(u8),   // "load" + Reg
    Creg(u8),   // "creg" + Reg
    Vreal(u8),  // "vreal" + Reg
    Vcplx(u8),  // "vcplx" + Reg
    Vsave(u8),  // "vsave" + Reg
    Vload(u8),  // "vload" + Reg

    Dup,       // "dup"
    Drop,      // "drop"
    Over,      // "over"
    Rot,       // "rot"
    Swap,      // "swap"
    Clear,     // "clear"
    Clregs,    // "clregs"
    DumpStack, // "dumpstack" | "ds"
    DumpReg,   // "dumpreg" | "dr"
    DumpVec,   // "dumpvec" | "dv"

    Add, // "+" | "add"
    Sub, // "-" | "sub"
    Mul, // "*" | "mul"
    Div, // "/" | "div"
    And, // "and"
    Or,  // "or"
    Xor, // "xor"
    Neg, // "neg"
    Shl, // "shl"
    Shr, // "shr"

    Abs,   // "abs"
    Floor, // "floor"
    Ceil,  // "ceil"
    Round, // "round"
    CosR,  // "cosr"
    SinR,  // "sinr"
    TanR,  // "tanr"
    CosD,  // "cosd"
    SinD,  // "sind"
    TanD,  // "tand"
    AcosR, // "acosr"
    AsinR, // "asinr"
    AtanR, // "atanr"
    AcosD, // "acosd"
    AsinD, // "asind"
    AtanD, // "atand"
    Loge,  // "loge"
    Log2,  // "log2"
    Log10, // "log10"
    Logx,  // "logx"
    Expe,  // "expe"
    Exp2,  // "exp2"
    Exp10, // "exp10"
    Expx,  // "expx"
    Gt,    // ">"
    Lt,    // "<"
    Ge,    // ">="
    Le,    // "<="
    Eq,    // "="

    Real,         // "real"
    Imag,         // "imag"
    R2c,          // "r2c"
    Precision,    // "k" | "precision" => {
    GetPrecision, // "K"
    Print,        // "p" | "print"
    // Help,      // help() called in parser,
    Quit, // "quit" | "bye" | "exit" | "q"
}

pub fn help() {
    println!("RPN complex calculator, inspired by the FORTH, gforth and dc commands.");
    println!("Cmdline args: -q or --quiet, -f <filename> or --file <filename>, and -h or --help");
    println!();
    println!("   Basic example:      10 6 4 - / p                # p as print, 6 - 4 --> 2    10 / 2 = 5");
    println!();
    println!("   Stack operation:    dup drop over rot swap clear dumpstack(ds)");
    println!("   Stack <--> Reg:     saveX loadX     cregX clregs    # where X a letter");
    println!(
        "   Stack <--> Vector:  vsaveX and vloadX, and   LEN vrealX or LEN vcplxX for create."
    );
    println!("   Debug:              dumpstack(ds), dumpreg(dr), dumpvec(dv)");
    println!();
    println!("   Literal:            3 4j                        # real or complex number");
    println!("   Arithmetic:         + - * / abs");
    println!("   Rounding:           floor ceil round");
    println!("   Complex:            real imag r2c");
    println!("   Logical:            and or xor neg, N shl N shr");
    println!();
    println!("   Trigonometric(rad): sinr, cosr, tanr, asinr, acosr, atanr");
    println!("   Trigonometric(deg): sind, cosd, tand, asind, acosd, atand");
    println!("   Logarithm:          loge expe log10 exp10 log2 exp2 logx expx");
    println!();
    println!("   Output:             print or p                  # stack is unchanged!");
    println!("   Output precision:   4 k                         # N.xxxx, redefineable, default and max 17 (K)");
    println!();
    println!("   Subrutine:          : srname 10 4 p drop ;      # multiline is allowed.");
    println!("   Call subrutine:     srname                      # as a normal command label");
    println!();
    println!("   Relation:           5 4 > p                     # 1");
    println!("   Loop:               10 [ 1 - p dup ]            # loop if not 0 before ']' and pop the result");
    println!("   Loop:               10 [ 1 - p dup 5 > ]        # loop if greater than 5");
    println!();
    println!("   Quit:               q quit bye exit");
    println!();
}
