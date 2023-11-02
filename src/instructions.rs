#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    Literal(f64),
    Call(usize), // ":"
    Ret,         // ";"
    Jnz(usize),  // "]", jump back

    Dup,       // "dup"
    Drop,      // "drop"
    Over,      // "over"
    Rot,       // "rot"
    Swap,      // "swap"
    Clear,     // "clear"
    DumpStack, // "dumpstack" | "ds"

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
    /*
        Real, // "real"
        Imag, // "imag"
        R2c,  // "r2c"
    */
    // Registers
    Save(u8), // RNUM + "save"
    Load(u8), // RNUM + "load"
    //    Creg(u8), // RNUM + "creg"
    //    Clregs,   // "clregs"
    DumpReg, // "dumpreg" | "dr"

    // Vectors
    Vreal(u8), // VNUM + "vreal"
    //    Vcplx(u8), // VNUM + "vcplx"
    Vsave(u8), // VNUM + "vsave"
    Vload(u8), // VNUM + "vload"
    Cvec(u8),  // VNUM + "cvec"
    Clvecs,    // "clvecs"
    DumpVec,   // "dumpvec" | "dv"

    // Print
    FractionalDigit, // "frdigit" | "precision" => {
    Print,           // "p" | "print"
    // Help,      // help() called in parser,
    Quit, // "quit" | "bye" | "exit" | "q"
}

pub fn help() {
    println!("RPN >>REAL ONLY<< calculator, inspired by the FORTH, gforth and dc commands.");
    println!("Cmdline args: -q or --quiet, -f <filename> or --file <filename>, and -h or --help");
    println!();
    println!("   Basic example:      10 6 4 - / p                     # p as print, 6 - 4 --> 2    10 / 2 = 5");
    println!();
    println!("   Stack operation:    dup drop over rot swap clear");
    println!("   Stack <--> Reg:     RNUM save load creg              # registernumber is 8 bit");
    println!("   Stack <--> Vector:  VNUM vsave vload cvec            # VNUM is 8 bit");
    println!("   Create a vector:    LEN VNUM vreal                   # VNUM is 8 bit");
    println!();
    println!("   Clear reg and vec:  NUM creg NUM vreg                # hide on debug");
    println!("   Debug:              dumpstack(ds), dumpreg(dr), dumpvec(dv)");
    println!();
    println!("   Arithmetic:         + - * / abs");
    println!("   Rounding:           floor ceil round");
    println!("   Logical:            and or xor neg, N shl N shr");
    println!();
    println!("   Trigonometric(rad): sinr, cosr, tanr, asinr, acosr, atanr");
    println!("   Trigonometric(deg): sind, cosd, tand, asind, acosd, atand");
    println!("   Logarithm:          loge expe log10 exp10 log2 exp2 logx expx");
    println!();
    println!("   Output:             print or p                       # stack is unchanged!");
    println!(
        "   Output frac. digit: 4 frdigit                        # N.xxxx, 0 auto, max 17 (K)"
    );
    println!();
    println!("   Subroutine:         : srname 10 4 p drop ;           # multiline is allowed.");
    println!("   Call subroutine:    srname                           # as a normal command label");
    println!("   List subroutines:   dumpsr(dsr)                      # print first line");
    println!();
    println!("   Relation:           5 4 > p                          # 1");
    println!("   Loop:               10 [ 1 - p dup ]                 # loop if not 0 before ']' and pop the result");
    println!("   Loop:               10 [ 1 - p dup 5 > ]             # loop if greater than 5");
    println!();
    println!("   Quit:               q quit bye exit");
    println!();
}
