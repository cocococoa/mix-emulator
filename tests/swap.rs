use mix_emulator::asm::debug_assemble;
use mix_emulator::vm::MixVM;
use std::collections::HashMap;

#[test]
fn test_swap1() {
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
    let (entry_point, binary) = debug_assemble(code);
    // addressと行の対応表を作成
    let mut v = vec![];
    let mut table = HashMap::new();
    for (line, address, word) in binary {
        table.insert(address, line);
        println!("address: {:4}, BINARY: {}", address, word);
        // println!("{:2}, {:4}, {}", line, address, word);
        v.push((address, word));
    }

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

    let mut vm = MixVM::new();
    vm.load(&v);
    vm.set_pc(entry_point);
    vm.read(16, input1);
    vm.read(16, input2);
    let mut counter = vec![(0, 0); 4000];
    let mut current_clock = 0;
    loop {
        match vm.step() {
            Ok((pc, _inst)) => {
                let (times, clocks) = counter[pc];
                let tmp_clock = vm.clock();
                counter[pc] = (times + 1, clocks + (tmp_clock - current_clock));
                current_clock = tmp_clock;
            }
            Err(()) => {
                // REACH HLT
                break;
            }
        }
    }

    println!("clock: {}", vm.clock());
    println!("content: \n{}", vm.print(18));
    // 実行時情報を分析
    let code = split_by_line(code.to_string());
    println!("line                       code  times  clocks");
    let mut var_a = 0;
    let mut var_b = 0;
    let mut var_c = 0;
    let mut var_d = 0;
    let mut var_e = 0;
    let mut var_f = 0;
    let mut var_g = 0;
    let mut var_h = 0;
    let mut var_j = 0;
    let mut var_k = 0;
    let mut var_l = 0;
    let mut var_p = 0;
    let mut var_q = 0;
    let mut var_r = 0;
    let mut var_s = 0;
    for i in 0..4000 {
        let (times, clocks) = counter[i];
        if times > 0 {
            let address = i;
            let line = table.get(&address).unwrap();
            println!(
                "{:4}, {:>25}, {:5}, {:6}",
                line + 1,
                code.get(line).unwrap(),
                times,
                clocks
            );

            let line = line + 1;
            macro_rules! test {
                ($begin: expr, $end: expr, $var: expr) => {
                    if $begin <= line && line <= $end {
                        if $var != 0 {
                            assert_eq!(times, $var);
                        } else {
                            $var = times;
                        }
                    }
                };
            }
            test!(26, 28, var_a);
            test!(29, 32, var_b);
            test!(33, 24, var_c);
            test!(35, 35, var_d);
            test!(36, 38, var_c);
            test!(41, 41, var_e);
            test!(42, 43, var_f);
            test!(44, 46, var_g);
            test!(64, 66, var_h);
            test!(67, 70, var_j);
            test!(72, 74, var_q);
            test!(75, 76, var_k);
            test!(77, 79, var_l);
            test!(80, 81, var_p);
            test!(82, 84, var_r);
            test!(85, 86, var_s);
        }
    }
    // 1. Kirchhoff
    assert_eq!(var_a, var_c);
    assert_eq!(var_e, var_r + 1);
    assert_eq!(var_f, var_e + var_g - 1);
    assert_eq!(var_h, var_f - var_g);
    assert_eq!(var_j, var_h + var_k - (var_l - var_j));
    assert_eq!(var_k, var_q + var_l - var_p);
    assert_eq!(var_r, var_p - var_q);

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
}

#[test]
fn test_swap2() {
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

    let (entry_point, binary) = debug_assemble(code);
    // addressと行の対応表を作成
    let mut v = vec![];
    let mut table = HashMap::new();
    for (line, address, word) in binary {
        table.insert(address, line);
        println!("{:2}, {:4}, {}", line, address, word);
        v.push((address, word));
    }

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

    let mut vm = MixVM::new();
    vm.load(&v);
    vm.set_pc(entry_point);
    vm.read(16, input1);
    vm.read(16, input2);
    let mut counter = vec![(0, 0); 4000];
    let mut current_clock = 0;
    loop {
        match vm.step() {
            Ok((pc, _inst)) => {
                let (times, clocks) = counter[pc];
                let tmp_clock = vm.clock();
                counter[pc] = (times + 1, clocks + (tmp_clock - current_clock));
                current_clock = tmp_clock;
            }
            Err(()) => {
                // REACH HLT
                break;
            }
        }
    }

    println!("clock: {}", vm.clock());
    println!("content: \n{}", vm.print(18));

    // 実行時情報を分析
    let code = split_by_line(code.to_string());
    let mut var_a = 0;
    let mut var_b = 0;
    let mut var_c = 0;
    let mut var_d = 0;
    let mut var_e = 0;
    let mut var_f = 0;
    let mut var_g = 0;
    let mut var_h = 0;
    let mut var_j = 0;
    let mut var_k = 0;
    let mut var_l = 0;
    let mut var_p = 0;
    let mut var_q = 0;
    let mut var_r = 0;
    let mut var_s = 0;
    let mut var_t = 0;
    let mut var_w = 0;
    let mut var_z = 0;
    println!("line                       code  times  clocks");
    for i in 0..4000 {
        let (times, clocks) = counter[i];
        if times > 0 {
            let address = i;
            let line = table.get(&address).unwrap();
            println!(
                "{:4}, {:>25}, {:5}, {:6}",
                line + 1,
                code.get(line).unwrap(),
                times,
                clocks
            );

            let line = line + 1;
            macro_rules! test {
                ($begin: expr, $end: expr, $var: expr) => {
                    if $begin <= line && line <= $end {
                        if $var != 0 {
                            assert_eq!(times, $var);
                        } else {
                            $var = times;
                        }
                    }
                };
            }
            test!(27, 27, var_a);
            test!(28, 30, var_b);
            test!(31, 32, var_c);
            test!(33, 34, var_d);
            test!(35, 36, var_e);
            test!(37, 39, var_f);
            test!(40, 40, var_g);
            test!(41, 44, var_h);
            test!(45, 48, var_j);
            test!(49, 50, var_k);
            test!(51, 51, var_l);
            test!(52, 52, var_p);
            test!(56, 57, var_q);
            test!(58, 59, var_r);
            test!(60, 60, var_s);
            test!(61, 65, var_t);
            test!(66, 66, var_w);
            test!(67, 68, var_z);
        }
    }

    println!("content: \n{}", vm.print(18));
}

fn five(c: char) -> String {
    let mut ret = " ".repeat(2).to_string();
    ret.push(c);
    ret.push_str("  ");
    ret
}

fn split_by_line(code: String) -> HashMap<usize, String> {
    let mut ret = HashMap::new();

    for (l, content) in code.split_terminator('\n').enumerate() {
        ret.insert(l, content.trim().to_string());
    }

    ret
}
