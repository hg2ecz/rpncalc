# rpncalc

RPN complex calculator. Inspired by the FORTH, gforth and dc commands.
```
RPN complex calculator, inspired by the FORTH, gforth and dc commands.
Cmdline args: -q or --quiet, -f <filename> or --file <filename>, and -h or --help

   Basic example:      10 6 4 - / p                # p as print, 6 - 4 --> 2    10 / 2 = 5

   Stack operation:    dup drop over rot swap clear dumpstack(ds)
   Stack <--> Reg:     RNUM save load creg clregs  # registernumber is 8 bit
   Stack <--> Vector:  VNUM vsave vload and LEN VNUM vreal or LEN VNUM vcplx for create. VNUM is 8 bit.
   Debug:              dumpstack(ds), dumpreg(dr), dumpvec(dv)

   Literal:            3 4j                        # real or complex number
   Arithmetic:         + - * / abs
   Rounding:           floor ceil round
   Complex:            real imag r2c
   Logical:            and or xor neg, N shl N shr

   Trigonometric(rad): sinr, cosr, tanr, asinr, acosr, atanr
   Trigonometric(deg): sind, cosd, tand, asind, acosd, atand
   Logarithm:          loge expe log10 exp10 log2 exp2 logx expx

   Output:             print or p                  # stack is unchanged!
   Output precision:   4 k                         # N.xxxx, redefineable, default and max 17 (K)

   Subrutine:          : srname 10 4 p drop ;      # multiline is allowed.
   Call subrutine:     srname                      # as a normal command label

   Relation:           5 4 > p                     # 1
   Loop:               10 [ 1 - p dup ]            # loop if not 0 before ']' and pop the result
   Loop:               10 [ 1 - p dup 5 > ]        # loop if greater than 5

   Quit:               q quit bye exit
```
