use mix_emulator::asm::debug_assemble;
use mix_emulator::tools::debug_run;
// use mix_emulator::tools::run;
use mix_emulator::vm::MixVM;

#[test]
fn test_topsort() {
   // 1. make input
   let code = "* DEFINE
                COUNT EQU 2:3
                QLINK EQU 2:3
                TOP EQU 4:5
                SUC EQU 2:3
                NEXT EQU 4:5
                TAPEIN EQU 1
                TAPEOUT EQU 2
                BUFFER ORIG *+100
                CON -1
                * input phase
        TOPSORT IN BUFFER(TAPEIN)
                JBUS *(TAPEIN)
             1H LD6 BUFFER+1
                ENT4 0,6
                STZ X,4
                DEC4 1
                J4NN *-2
                ENT2 X,6
                ENT5 BUFFER+2
             2H LD3 0,5
                J3P 3F
                J3Z 4F
                IN BUFFER(TAPEIN)
                JBUS *(TAPEIN)
                ENT5 BUFFER
                JMP 2B
             3H LD4 1,5
                LDA X,4(COUNT)
                INCA 1
                STA X,4(COUNT)
                INC2 1
                LDA X,3(TOP)
                STA 0,2(NEXT)
                ST4 0,2(SUC)
                ST2 X,3(TOP)
                INC5 2
                JMP 2B
             4H IOC 0(TAPEIN)
                ENT4 0,6
                ENT5 -100
                ENT3 0
             4H LDA X,4(COUNT)
                JAP *+3
                ST4 X,3(QLINK)
                ENT3 0,4
                DEC4 1
                J4P 4B
                * sort
                LD1 X(QLINK)
             5H JBUS *(TAPEOUT)
                ST1 BUFFER+100,5
                J1Z 8F
                INC5 1
                J5N *+3
                OUT BUFFER(TAPEOUT)
                ENT5 -100
                DEC6 1
                LD2 X,1(TOP)
                J2Z 7F
             6H LD4 0,2(SUC)
                LDA X,4(COUNT)
                DECA 1
                STA X,4(COUNT)
                JAP *+3
                ST4 X,3(QLINK)
                ENT3 0,4
                LD2 0,2(NEXT)
                J2P 6B
             7H LD1 X,1(QLINK)
                JMP 5B
             8H OUT BUFFER(TAPEOUT)
                IOC 0(TAPEOUT)
                HLT 0,6
              X END TOPSORT";

   let mut input = vec![
      0, 10, 9, 2, 3, 7, 7, 5, 5, 8, 8, 6, 4, 6, 1, 3, 7, 4, 9, 5, 2, 8, 10, 1, 10, 6, 8, 4, 0, 0,
   ];
   loop {
      input.push(0);
      if input.len() == 100 {
         break;
      }
   }

   // 2. run VM
   let (entry_point, binary, _table) = debug_assemble(code);
   let mut vm = MixVM::new();
   vm.read_binary(1, input);
   vm.load(&binary);
   vm.set_pc(entry_point);
   let (_inst, _runinfo) = debug_run(&mut vm).unwrap();
   // let runinfo = run(&mut vm).unwrap();

   // 3. analyze run information
   // assert_eq!(
   //    runinfo.count_clocks(table[&(10 - 1)], table[&(24 - 1)]),
   //    Some(182144)
   // );
   // assert_eq!(runinfo.count_exec(table[&(19 - 1)]), 9538);

   // 4. OMAKE
   println!("[test: topsort]\n{:?}", vm.print_binary(2));
}
