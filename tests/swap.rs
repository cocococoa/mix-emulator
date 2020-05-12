use mix_emulator::asm::debug_assemble;
use mix_emulator::tools::run;
use mix_emulator::vm::MixVM;

#[test]
fn test_swap1() {
    // 1. make input
    let code = "MAXWDS EQU 1200
                PERM ORIG *+MAXWDS
                ANS ORIG *+MAXWDS
                OUTBUF ORIG *+24
                CARDS EQU 16
                PRINTER EQU 18
                BEGIN IN PERM(CARDS)
                ENT2 0
                LDA EQUALS
                1H JBUS *(CARDS)
                CMPA PERM+15,2
                JE *+2
                IN PERM+16,2(CARDS)
                ENT1 OUTBUF
                JBUS *(PRINTER)
                MOVE PERM,2(16)
                OUT OUTBUF(PRINTER)
                JE 1F
                INC2 16
                CMP2 =MAXWDS-16=
                JLE 1B
                HLT 666
                1H INC2 15
                ST2 SIZE
                ENT3 0
                2H LDAN PERM,3
                CMPA LPREN(1:5)
                JNE 1F
                STA PERM,3
                INC3 1
                LDXN PERM,3
                JXZ *-2
                1H CMPA RPREN(1:5)
                JNE *+2
                STX PERM,3
                INC3 1
                CMP3 SIZE
                JL 2B
                LDA LPREN
                ENT1 ANS
                OPEN ENT3 0
                1H LDXN PERM,3
                JXN G0
                INC3 1
                CMP3 SIZE
                JL 1B
                *
                DONE CMP1 =ANS=
                JNE *+2
                MOVE LPREN(2)
                MOVE =0=
                MOVE -1,1(22)
                ENT3 0
                OUT ANS,3(PRINTER)
                INC3 24
                LDX ANS,3
                JXNZ *-3
                HLT
                *
                LPREN ALF ____(
                RPREN ALF )____
                EQUALS ALF ____=
                *
                G0 MOVE LPREN
                MOVE PERM,3
                STX START
                SUCC STX PERM,3
                INC3 1
                LDXN PERM,3(1:5)
                JXN 1F
                JMP *-3
                5H STX 0,1
                INC1 1
                ENT3 0
                4H CMPX PERM,3(1:5)
                JE SUCC
                1H INC3 1
                CMP3 SIZE
                JL 4B
                CMPX START(1:5)
                JNE 5B
                CLOSE MOVE RPREN
                CMPA -3,1
                JNE OPEN
                INC1 -3
                JMP OPEN
                SIZE CON 0
                START CON 0
                END BEGIN";
    let input1 = vec![
        "    (".to_string(),
        five('A'),
        five('C'),
        five('F'),
        five('G'),
        ")    ".to_string(),
        "    (".to_string(),
        five('B'),
        five('C'),
        five('D'),
        ")    ".to_string(),
        "    (".to_string(),
        five('A'),
        five('E'),
        five('D'),
        ")    ".to_string(),
    ];
    let input2 = vec![
        "    (".to_string(),
        five('F'),
        five('A'),
        five('D'),
        five('E'),
        ")    ".to_string(),
        "    (".to_string(),
        five('B'),
        five('G'),
        five('F'),
        five('A'),
        five('E'),
        ")    ".to_string(),
        five(' '),
        five(' '),
        "    =".to_string(),
    ];

    // 2. run VM
    let (entry_point, binary, table) = debug_assemble(code);
    let mut vm = MixVM::new();
    vm.load(&binary);
    vm.set_pc(entry_point);
    vm.read(16, input1);
    vm.read(16, input2);
    let runinfo = run(&mut vm).unwrap();

    // 3. analyze run information
    macro_rules! test {
        ($begin: expr, $end: expr, $var: expr) => {
            for i in $begin..=$end {
                assert_eq!($var, runinfo.count_exec(table[&(i - 1)]));
            }
        };
    }
    let var_a = runinfo.count_exec(table[&(26 - 1)]);
    test!(26, 28, var_a);
    let var_b = runinfo.count_exec(table[&(29 - 1)]);
    test!(29, 32, var_b);
    let var_c = runinfo.count_exec(table[&(33 - 1)]);
    test!(33, 34, var_c);
    let var_d = runinfo.count_exec(table[&(35 - 1)]);
    test!(35, 35, var_d);
    test!(36, 38, var_c);
    let var_e = runinfo.count_exec(table[&(41 - 1)]);
    test!(41, 41, var_e);
    let var_f = runinfo.count_exec(table[&(42 - 1)]);
    test!(42, 43, var_f);
    let var_g = runinfo.count_exec(table[&(44 - 1)]);
    test!(44, 46, var_g);
    let var_h = runinfo.count_exec(table[&(64 - 1)]);
    test!(64, 66, var_h);
    let var_j = runinfo.count_exec(table[&(67 - 1)]);
    test!(67, 70, var_j);
    let var_q = runinfo.count_exec(table[&(72 - 1)]);
    test!(72, 74, var_q);
    let var_k = runinfo.count_exec(table[&(75 - 1)]);
    test!(75, 76, var_k);
    let var_l = runinfo.count_exec(table[&(77 - 1)]);
    test!(77, 79, var_l);
    let var_p = runinfo.count_exec(table[&(80 - 1)]);
    test!(80, 81, var_p);
    let var_r = runinfo.count_exec(table[&(82 - 1)]);
    test!(82, 84, var_r);
    let var_s = runinfo.count_exec(table[&(85 - 1)]);
    test!(85, 86, var_s);
    // 3-1. Kirchhoff
    assert_eq!(var_a, var_c);
    assert_eq!(var_e, var_r + 1);
    assert_eq!(var_f, var_e + var_g - 1);
    assert_eq!(var_h, var_f - var_g);
    assert_eq!(var_j, var_h + var_k - (var_l - var_j));
    assert_eq!(var_k, var_q + var_l - var_p);
    assert_eq!(var_r, var_p - var_q);
    // 3-2. other equation
    let var_x = 2;
    let var_y = 29; // is this 30??
    assert_eq!(var_b + var_c, 16 * var_x - 1);
    assert_eq!(var_b, 5);
    assert_eq!(var_d, 5);
    assert_eq!(var_j, var_y - 2 * var_b);
    assert_eq!(var_p, var_h + var_q);
    assert_eq!(var_p, 7); // a,b,c,d,e,f,g -> 7
    assert_eq!(var_s, 1); // (f) -> 1
    assert_eq!(var_g + var_j + var_l, (var_b + var_c) * (var_p + 1));

    // 4. OMAKE
    println!("[test: swap1]\n{}", vm.print(18));
}

#[test]
fn test_swap2() {
    // 1. make input
    let code = "MAXWDS EQU 1200
                X ORIG *+MAXWDS
                T ORIG *+MAXWDS
                PERM ORIG *+MAXWDS
                ANS EQU 2*MAXWDS
                OUTBUF ORIG *+24
                CARDS EQU 16
                PRINTER EQU 18
                BEGIN IN PERM(CARDS)
                ENT2 0
                LDA EQUALS
                1H JBUS *(CARDS)
                CMPA PERM+15,2
                JE *+2
                IN PERM+16,2(CARDS)
                ENT1 OUTBUF
                JBUS *(PRINTER)
                MOVE PERM,2(16)
                OUT OUTBUF(PRINTER)
                JE 1F
                INC2 16
                CMP2 =MAXWDS-16=
                JLE 1B
                HLT 666
                1H INC2 15
                ENT3 0
                RIGHT ENTX 0
                SCAN DEC2 1
                LDA PERM,2
                JAZ CYCLE
                CMPA RPREN
                JE RIGHT
                CMPA LPREN
                JE LEFT
                ENT4 1,3
                STA X
                2H DEC4 1
                CMPA X,4
                JNE 2B
                J4P FOUND
                INC3 1
                STA X,3
                ST3 T,3
                ENT4 0,3
                FOUND LDA T,4
                STX T,4
                SRC 5
                JANZ SCAN
                ENT1 0,4
                JMP SCAN
                LEFT STX T,1
                CYCLE J2P SCAN
                *
                OUTPUT ENT1 ANS
                J3Z DONE
                1H LDAN X,3
                JAP SKIP
                CMP3 T,3
                JE SKIP
                MOVE LPREN
                2H MOVE X,3
                STA X,3
                LD3 T,3
                LDAN X,3
                JAN 2B
                MOVE RPREN
                SKIP DEC3 1
                J3P 1B
                *
                DONE CMP1 =ANS=
                JNE *+2
                MOVE LPREN(2)
                MOVE =0=
                MOVE -1,1(22)
                ENT3 0
                OUT ANS,3(PRINTER)
                INC3 24
                LDX ANS,3
                JXNZ *-3
                HLT
                *
                LPREN ALF ____(
                RPREN ALF )____
                EQUALS ALF ____=
                END BEGIN";

    let input1 = vec![
        "    (".to_string(),
        five('A'),
        five('C'),
        five('F'),
        five('G'),
        ")    ".to_string(),
        "    (".to_string(),
        five('B'),
        five('C'),
        five('D'),
        ")    ".to_string(),
        "    (".to_string(),
        five('A'),
        five('E'),
        five('D'),
        ")    ".to_string(),
    ];
    let input2 = vec![
        "    (".to_string(),
        five('F'),
        five('A'),
        five('D'),
        five('E'),
        ")    ".to_string(),
        "    (".to_string(),
        five('B'),
        five('G'),
        five('F'),
        five('A'),
        five('E'),
        ")    ".to_string(),
        five(' '),
        five(' '),
        "    =".to_string(),
    ];

    // 2. run VM
    let (entry_point, binary, table) = debug_assemble(code);
    let mut vm = MixVM::new();
    vm.load(&binary);
    vm.set_pc(entry_point);
    vm.read(16, input1);
    vm.read(16, input2);
    let runinfo = run(&mut vm).unwrap();

    // 3. analyze run information
    macro_rules! test {
        ($begin: expr, $end: expr, $var: expr) => {
            for i in $begin..=$end {
                assert_eq!($var, runinfo.count_exec(table[&(i - 1)]));
            }
        };
    }
    let var_a = runinfo.count_exec(table[&(27 - 1)]);
    test!(27, 27, var_a);
    let var_b = runinfo.count_exec(table[&(28 - 1)]);
    test!(28, 30, var_b);
    let var_c = runinfo.count_exec(table[&(31 - 1)]);
    test!(31, 32, var_c);
    let var_d = runinfo.count_exec(table[&(33 - 1)]);
    test!(33, 34, var_d);
    let var_e = runinfo.count_exec(table[&(35 - 1)]);
    test!(35, 36, var_e);
    let var_f = runinfo.count_exec(table[&(37 - 1)]);
    test!(37, 39, var_f);
    let var_g = runinfo.count_exec(table[&(40 - 1)]);
    test!(40, 40, var_g);
    let var_h = runinfo.count_exec(table[&(41 - 1)]);
    test!(41, 44, var_h);
    let var_j = runinfo.count_exec(table[&(45 - 1)]);
    test!(45, 48, var_j);
    let var_k = runinfo.count_exec(table[&(49 - 1)]);
    test!(49, 50, var_k);
    let var_l = runinfo.count_exec(table[&(51 - 1)]);
    test!(51, 51, var_l);
    let var_p = runinfo.count_exec(table[&(52 - 1)]);
    test!(52, 52, var_p);
    let var_q = runinfo.count_exec(table[&(56 - 1)]);
    test!(56, 57, var_q);
    let var_r = runinfo.count_exec(table[&(58 - 1)]);
    test!(58, 59, var_r);
    let var_s = runinfo.count_exec(table[&(60 - 1)]);
    test!(60, 60, var_s);
    let var_t = runinfo.count_exec(table[&(61 - 1)]);
    test!(61, 65, var_t);
    let var_w = runinfo.count_exec(table[&(66 - 1)]);
    test!(66, 66, var_w);
    let var_z = runinfo.count_exec(table[&(67 - 1)]);
    test!(67, 68, var_z);

    // 4. OMAKE
    println!("[test: swap2]\n{}", vm.print(18));
}

fn five(c: char) -> String {
    let mut ret = " ".repeat(2).to_string();
    ret.push(c);
    ret.push_str("  ");
    ret
}
