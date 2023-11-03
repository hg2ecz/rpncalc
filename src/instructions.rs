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

    // Registers
    Save(u8), // RNUM + "save"
    Load(u8), // RNUM + "load"
    //Clreg(u8), // RNUM + "creg"
    //Clregs,   // "clregs"
    DumpReg, // "dumpreg" | "dr"

    // Vectors
    Vcreate(u8), // VNUM + "vreal"
    Vsave(u8),   // VNUM + "vsave"
    Vload(u8),   // VNUM + "vload"
    Cvec(u8),    // VNUM + "cvec"
    Clvecs,      // "clvecs"
    DumpVec,     // "dumpvec" | "dv"

    // Print
    FractionalDigit, // "frdigit" | "precision" => {
    Print,           // "p" | "print"

    // === Complex ===
    CplxReal, // "real" cf64 -> f64
    CplxImag, // "imag" cf64 -> f64
    CplxR2c,  // "r2c"  (f64, f64) -> cf64
    CplxC2r,  // "c2r"  cf64 -> (f64, f64)

    CplxDup,       // "dup"
    CplxDrop,      // "drop"
    CplxOver,      // "over"
    CplxRot,       // "rot"
    CplxSwap,      // "swap"
    CplxClear,     // "clear"
    CplxDumpStack, // "dumpstack" | "ds"

    CplxAdd,
    CplxSub,
    CplxMul,
    CplxDiv,
    CplxAbs, // cf64 -> f64

    CplxSave(u8),
    CplxLoad(u8),
    CplxDumpReg,

    CplxVcreate(u8), // VNUM + "vreal"
    CplxVsave(u8),   // VNUM + "vsave"
    CplxVload(u8),   // VNUM + "vload"
    CplxCvec(u8),    // VNUM + "cvec"
    CplxClvecs,      // "clvecs"
    CplxDumpVec,     // "dumpvec" | "dv"

    CplxPrint,

    // Help,      // help() called in parser,
    Quit, // "quit" | "bye" | "exit" | "q"
}

pub fn help() {
    println!("RPN complex calculator, inspired by the FORTH, gforth and dc commands.");
    println!("Cmdline args: -q or --quiet, -f <filename> or --file <filename>, and -h or --help");
    println!();
    println!("   Basic example:      10 6 4 - / p                     # p as print, 6 - 4 --> 2    10 / 2 = 5");
    println!();
    println!("   Stack operation:    dup drop over rot swap clear");
    println!("   Stack <--> Reg:     RNUM save load creg              # registernumber is 8 bit");
    println!("   Stack <--> Vector:  VNUM vsave vload cvec            # VNUM is 8 bit");
    println!("   Create a vector:    LEN VNUM vcreate                 # VNUM is 8 bit");
    println!();
    println!("   Clear reg and vec:  NUM cvec, clvecs");
    println!("   Debug:              dumpstack or ds, dumpreg or dr, dumpvec or dv");
    println!();
    println!("   Literal:            3 4j                             # real or complex number");
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
    println!("   Complex:            r2c cadd csub cdiv cabs r2c creal cimag");
    println!("   Stack operation:    cdup cdrop cover crot cswap cclear");
    println!("   Clear reg and vec:  NUM ccreg NUM cvreg, cclregs cclvecs # hide on debug");
    println!(
        "   Stack <--> Reg:     RNUM csave cload ccreg              # registernumber is 8 bit"
    );
    println!("   Stack <--> Vector:  VNUM cvsave cvload ccvec            # VNUM is 8 bit");
    println!("   Create a vector:    LEN VNUM cvcreate                # VNUM is 8 bit");
    println!("   Clear reg and vec:  NUM ccvec, cclvecs");
    println!("   Debug:              cdumpstack or cds, cdumpreg or cdr, cdumpvec or cdv");
    println!("   Output:             cprint or cp                     # stack is unchanged!");

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
