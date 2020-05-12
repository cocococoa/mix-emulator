use mix_emulator::asm::debug_assemble;
use mix_emulator::tools::run;
use mix_emulator::vm::MixVM;

#[test]
fn test_prime500() {
    // 1. make input
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

    // 2. run VM
    let (entry_point, binary, table) = debug_assemble(code);
    let mut vm = MixVM::new();
    vm.load(&binary);
    vm.set_pc(entry_point);
    let runinfo = run(&mut vm).unwrap();

    // 3. analyze run information
    assert_eq!(
        runinfo.count_clocks(table[&(10 - 1)], table[&(24 - 1)]),
        Some(182144)
    );
    assert_eq!(runinfo.count_exec(table[&(19 - 1)]), 9538);

    // 4. OMAKE
    println!("[test: prime500]\n{}", vm.print(18));
}
