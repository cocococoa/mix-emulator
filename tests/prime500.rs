use mix_emulator::asm::debug_assemble;
use mix_emulator::vm::MixVM;
use std::collections::HashMap;

#[test]
fn test_prime500() {
    let code = "* EXAMPLE PROGRAM ... TABLE OF PRIMES
                * 
                L EQU 500
                PRINTER EQU 18
                PRIME EQU -1
                BUF0 EQU 2000
                BUF1 EQU BUF0+25
                ORIG 3000
                START IOC 0(PRINTER)
                LD1 =1-L=
                LD2 =3=
                2H INC1 1
                ST2 PRIME+L,1
                J1Z 2F
                4H INC2 2
                ENT3 2
                6H ENTA 0
                ENTX 0,2
                DIV PRIME,3
                JXZ 4B
                CMPA PRIME,3
                INC3 1
                JG 6B
                JMP 2B
                WIDTH EQU 20
                2H OUT TITLE(PRINTER)
                ENT4 2000+WIDTH
                ENT5 -500/WIDTH
                2H INC5 L+1
                4H LDA PRIME,5
                CHAR
                STX 0,4(1:4)
                DEC4 1
                DEC5 500/WIDTH
                J5P 4B
                OUT 0,4(PRINTER)
                LD4 24,4
                J5N 2B
                HLT
                * INITIAL CONTENTS OF TABLES AND BUFFERS
                ORIG PRIME+1
                CON 2
                ORIG BUF0-5
                TITLE ALF FISRT
                ALF _FIVE
                ALF _HUND
                ALF RED_P
                ALF RIMES
                ORIG BUF0+24
                CON BUF1+WIDTH
                ORIG BUF1+24
                CON BUF0+WIDTH
                END START";
    let (entry_point, binary) = debug_assemble(code);
    // addressと行の対応表を作成
    let mut v = vec![];
    let mut table = HashMap::new();
    for (line, address, word) in binary {
        table.insert(address, line);
        println!("{:2}, {:4}, {}", line, address, word);
        v.push((address, word));
    }
    let mut vm = MixVM::new();
    vm.load(&v);
    vm.set_pc(entry_point);
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
    let mut clocks_sum = 0;
    for i in 0..4000 {
        let (times, clocks) = counter[i];
        if times > 0 {
            let address = i;
            let line = table.get(&address).unwrap();
            if 10 <= line + 1 && line + 1 <= 24 {
                clocks_sum += clocks;
            }
            if line + 1 == 19 {
                assert_eq!(9538, times);
            }
            println!(
                "{:4}, {:>25}, {:5}, {:6}",
                line + 1,
                code.get(line).unwrap(),
                times,
                clocks
            );
        }
    }
    assert_eq!(182144, clocks_sum);
}

fn split_by_line(code: String) -> HashMap<usize, String> {
    let mut ret = HashMap::new();

    for (l, content) in code.split_terminator('\n').enumerate() {
        ret.insert(l, content.trim().to_string());
    }

    ret
}
