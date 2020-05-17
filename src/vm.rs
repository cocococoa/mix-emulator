use crate::instruction;
use crate::iounit;
use crate::iounit::IOUnit;
use crate::mix_word::{Memory, Register, Sign, WordImpl};
use std::cmp::Ordering;

pub struct MixVM {
    // utility
    pc: usize,
    clock: i64,
    // onboard
    reg_a: Register,
    reg_x: Register,
    reg_i: Vec<Register>,
    reg_j: Register,
    overflow: bool,
    comp: Ordering,
    // memory
    memory: Memory,
    // IOUnit
    tape: Vec<iounit::Tape>,
    disk: Vec<iounit::Disk>,
    card_reader: iounit::CardReader,
    card_punch: iounit::CardPunch,
    line_printer: iounit::LinePrinter,
    type_writer_terminal: iounit::TypeWriterTerminal,
    paper_tape: iounit::PaperTape,
}

impl MixVM {
    pub fn new() -> Self {
        MixVM {
            pc: 0,
            clock: 0,
            reg_a: Register::from_val(0),
            reg_x: Register::from_val(0),
            reg_i: vec![Register::from_val(0); 6],
            reg_j: Register::from_val(0),
            memory: vec![Register::from_val(0); 4000],
            overflow: false,
            comp: Ordering::Equal,
            tape: vec![iounit::Tape::default(); 8],
            disk: vec![iounit::Disk::default(); 8],
            card_reader: iounit::CardReader::default(),
            card_punch: iounit::CardPunch::default(),
            line_printer: iounit::LinePrinter::default(),
            type_writer_terminal: iounit::TypeWriterTerminal::default(),
            paper_tape: iounit::PaperTape::default(),
        }
    }
    pub fn load(&mut self, code: &Vec<(usize, WordImpl)>) {
        for (address, word) in code {
            self.memory[*address] = word.clone();
        }
    }
    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }
    pub fn clock(&self) -> i64 {
        self.clock
    }
    pub fn print(&self, f: usize) -> String {
        // print out
        match f {
            0..=7 => self.tape[f].print(),
            8..=15 => self.disk[f - 8].print(),
            16 => self.card_reader.print(),
            17 => self.card_punch.print(),
            18 => self.line_printer.print(),
            19 => self.type_writer_terminal.print(),
            20 => self.paper_tape.print(),
            _ => unreachable!(),
        }
    }
    pub fn print_binary(&self, f: usize) -> Vec<i64> {
        // print out
        match f {
            0..=7 => self.tape[f].print_binary(),
            8..=15 => self.disk[f - 8].print_binary(),
            16 => self.card_reader.print_binary(),
            17 => self.card_punch.print_binary(),
            18 => self.line_printer.print_binary(),
            19 => self.type_writer_terminal.print_binary(),
            20 => self.paper_tape.print_binary(),
            _ => unreachable!(),
        }
    }
    pub fn read(&mut self, f: usize, i: Vec<String>) {
        match f {
            0..=7 => self.tape[f].set_input(i),
            8..=15 => self.disk[f - 8].set_input(i),
            16 => self.card_reader.set_input(i),
            17 => self.card_punch.set_input(i),
            18 => self.line_printer.set_input(i),
            19 => self.type_writer_terminal.set_input(i),
            20 => self.paper_tape.set_input(i),
            _ => unreachable!(),
        }
    }
    pub fn read_binary(&mut self, f: usize, i: Vec<i64>) {
        match f {
            0..=7 => self.tape[f].set_input_binary(i),
            8..=15 => self.disk[f - 8].set_input_binary(i),
            16 => self.card_reader.set_input_binary(i),
            17 => self.card_punch.set_input_binary(i),
            18 => self.line_printer.set_input_binary(i),
            19 => self.type_writer_terminal.set_input_binary(i),
            20 => self.paper_tape.set_input_binary(i),
            _ => unreachable!(),
        }
    }
    pub fn step(&mut self) -> Result<(usize, WordImpl), ()> {
        // 1. fetch
        let inst = self.memory[self.pc].clone();
        let current_pc = self.pc;
        let a = inst.address(); // address
        let i = inst.index(); // index
        let f = inst.modification() as usize; // modification
        let c = inst.operation(); // operation
        let m = if i == 0 {
            a
        } else {
            a + &self.reg_i[(i - 1) as usize].val()
        }; // modified address

        // println!(
        //     "line: {}, a: {:4}, i: {:2}, f: {:2}, c: {:2}, m: {:4}",
        //     current_pc - 2417,
        //     a,
        //     i,
        //     f,
        //     c,
        //     m
        // );
        // println!("\t rA: {}\n\t rX: {}\n\trI1: {}\n\trI2: {}\n\trI3: {}\n\trI4: {}\n\trI5: {}\n\trI6: {}\n\t rJ: {}", self.reg_a, self.reg_x, self.reg_i[0], self.reg_i[1], self.reg_i[2], self.reg_i[3], self.reg_i[4], self.reg_i[5], self.reg_j);
        // println!("\toverflow: {}, comp: {:?}", self.overflow, self.comp);

        // 2. define macros
        macro_rules! forward {
            ($pc: expr, $clock: expr) => {
                self.pc += $pc;
                self.clock += $clock as i64;
            };
        }
        macro_rules! jump {
            () => {
                self.reg_j = Register::from_val((self.pc + 1) as i64);
                self.pc = m as usize;
            };
        }
        macro_rules! macro40_47 {
            ($reg: expr) => {
                match f {
                    0 => {
                        // N
                        if $reg.val() < 0 {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    1 => {
                        // Z
                        if $reg.val() == 0 {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    2 => {
                        // P
                        if $reg.val() > 0 {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    3 => {
                        // NN
                        if $reg.val() >= 0 {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    4 => {
                        // NZ
                        if $reg.val() != 0 {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    5 => {
                        // NP
                        if $reg.val() <= 0 {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    _ => {
                        unreachable!();
                    }
                }
            };
        }
        macro_rules! macro48_55 {
            ($reg: expr) => {
                match f {
                    0 => {
                        instruction::inc(m as i64, $reg, &mut self.overflow);
                    }
                    1 => {
                        instruction::dec(m as i64, $reg, &mut self.overflow);
                    }
                    2 => {
                        let sign = if m == 0 {
                            *inst.sign()
                        } else if m > 0 {
                            Sign::Positive
                        } else {
                            Sign::Negative
                        };
                        instruction::ent(m as i64, &sign, $reg);
                    }
                    3 => {
                        let sign = if m == 0 {
                            *inst.sign()
                        } else if m > 0 {
                            Sign::Positive
                        } else {
                            Sign::Negative
                        };
                        instruction::entn(m as i64, &sign, $reg);
                    }
                    _ => {
                        unreachable!();
                    }
                }
            };
        }

        // 3. execute operation
        match c {
            0 => {
                // NOP
                forward!(1, 1);
            }
            1 => {
                // ADD
                instruction::add(
                    &self.memory[m as usize],
                    &mut self.reg_a,
                    f,
                    &mut self.overflow,
                );
                forward!(1, 2);
            }
            2 => {
                // SUB
                instruction::sub(
                    &self.memory[m as usize],
                    &mut self.reg_a,
                    f,
                    &mut self.overflow,
                );
                forward!(1, 2);
            }
            3 => {
                // MUL
                instruction::mul(
                    &self.memory[m as usize],
                    &mut self.reg_a,
                    &mut self.reg_x,
                    f,
                );
                forward!(1, 10);
            }
            4 => {
                // DIV
                instruction::div(
                    &self.memory[m as usize],
                    &mut self.reg_a,
                    &mut self.reg_x,
                    f,
                    &mut self.overflow,
                );
                forward!(1, 12);
            }
            5 => {
                // NUM/CHAR/HLT
                match f {
                    0 => {
                        // NUM
                        instruction::to_num(&mut self.reg_a, &self.reg_x, &mut self.overflow);
                    }
                    1 => {
                        // CHAR
                        instruction::to_char(&mut self.reg_a, &mut self.reg_x);
                    }
                    2 => {
                        // HLT
                        // TODO: 何をやる？
                        return Err(());
                    }
                    _ => {
                        unreachable!();
                    }
                }
                forward!(1, 10);
            }
            6 => {
                // SLA/SRA/SLAX/SRAX/SLC/SRC
                match f {
                    0 => {
                        instruction::shift_left(m as i64, &mut self.reg_a);
                    }
                    1 => {
                        instruction::shift_right(m as i64, &mut self.reg_a);
                    }
                    2 => {
                        instruction::shift_left_pair(m as i64, &mut self.reg_a, &mut self.reg_x);
                    }
                    3 => {
                        instruction::shift_right_pair(m as i64, &mut self.reg_a, &mut self.reg_x);
                    }
                    4 => {
                        instruction::rotate_left_pair(m as i64, &mut self.reg_a, &mut self.reg_x);
                    }
                    5 => {
                        instruction::rotate_right_pair(m as i64, &mut self.reg_a, &mut self.reg_x);
                    }
                    _ => {
                        unreachable!();
                    }
                }
                forward!(1, 2);
            }
            7 => {
                // MOVE
                instruction::mov(m as i64, &mut self.reg_i[0], f as i64, &mut self.memory);
                forward!(1, 1 + 2 * f);
            }
            8 => {
                // LDA
                instruction::load(&self.memory[m as usize], &mut self.reg_a, f);
                forward!(1, 2);
            }
            9 => {
                // LD1
                instruction::load(&self.memory[m as usize], &mut self.reg_i[0], f);
                forward!(1, 2);
            }
            10 => {
                // LD2
                instruction::load(&self.memory[m as usize], &mut self.reg_i[1], f);
                forward!(1, 2);
            }
            11 => {
                // LD3
                instruction::load(&self.memory[m as usize], &mut self.reg_i[2], f);
                forward!(1, 2);
            }
            12 => {
                // LD4
                instruction::load(&self.memory[m as usize], &mut self.reg_i[3], f);
                forward!(1, 2);
            }
            13 => {
                // LD5
                instruction::load(&self.memory[m as usize], &mut self.reg_i[4], f);
                forward!(1, 2);
            }
            14 => {
                // LD6
                instruction::load(&self.memory[m as usize], &mut self.reg_i[5], f);
                forward!(1, 2);
            }
            15 => {
                // LDX
                instruction::load(&self.memory[m as usize], &mut self.reg_x, f);
                forward!(1, 2);
            }
            16 => {
                // LDAN
                instruction::loadn(&self.memory[m as usize], &mut self.reg_a, f);
                forward!(1, 2);
            }
            17 => {
                // LD1N
                instruction::loadn(&self.memory[m as usize], &mut self.reg_i[0], f);
                forward!(1, 2);
            }
            18 => {
                // LD2N
                instruction::loadn(&self.memory[m as usize], &mut self.reg_i[1], f);
                forward!(1, 2);
            }
            19 => {
                // LD3N
                instruction::loadn(&self.memory[m as usize], &mut self.reg_i[2], f);
                forward!(1, 2);
            }
            20 => {
                // LD4N
                instruction::loadn(&self.memory[m as usize], &mut self.reg_i[3], f);
                forward!(1, 2);
            }
            21 => {
                // LD5N
                instruction::loadn(&self.memory[m as usize], &mut self.reg_i[4], f);
                forward!(1, 2);
            }
            22 => {
                // LD6N
                instruction::loadn(&self.memory[m as usize], &mut self.reg_i[5], f);
                forward!(1, 2);
            }
            23 => {
                // LDXN
                instruction::loadn(&self.memory[m as usize], &mut self.reg_x, f);
                forward!(1, 2);
            }
            24 => {
                // STA
                instruction::store(&self.reg_a, &mut self.memory[m as usize], f);
                forward!(1, 2);
            }
            25 => {
                // ST1
                instruction::store(&self.reg_i[0], &mut self.memory[m as usize], f);
                forward!(1, 2);
            }
            26 => {
                // ST2
                instruction::store(&self.reg_i[1], &mut self.memory[m as usize], f);
                forward!(1, 2);
            }
            27 => {
                // ST3
                instruction::store(&self.reg_i[2], &mut self.memory[m as usize], f);
                forward!(1, 2);
            }
            28 => {
                // ST4
                instruction::store(&self.reg_i[3], &mut self.memory[m as usize], f);
                forward!(1, 2);
            }
            29 => {
                // ST5
                instruction::store(&self.reg_i[4], &mut self.memory[m as usize], f);
                forward!(1, 2);
            }
            30 => {
                // ST6
                instruction::store(&self.reg_i[5], &mut self.memory[m as usize], f);
                forward!(1, 2);
            }
            31 => {
                // STX
                instruction::store(&self.reg_x, &mut self.memory[m as usize], f);
                forward!(1, 2);
            }
            32 => {
                // STJ
                instruction::store(&self.reg_j, &mut self.memory[m as usize], f);
                forward!(0, 2);
            }
            33 => {
                // STZ
                instruction::store_zero(&mut self.memory[m as usize]);
                forward!(1, 2);
            }
            34 => {
                // JBUS
                match f {
                    0..=7 => {
                        if self.tape[f].busy() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    8..=15 => {
                        if self.disk[f].busy() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    16 => {
                        if self.card_reader.busy() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    17 => {
                        if self.card_punch.busy() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    18 => {
                        if self.line_printer.busy() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    19 => {
                        if self.type_writer_terminal.busy() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    20 => {
                        if self.paper_tape.busy() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    _ => {
                        unreachable!();
                    }
                }
                forward!(0, 1);
            }
            35 => {
                // IOC
                match f {
                    0..=7 => {
                        while self.tape[f].busy() {
                            forward!(0, 1);
                        }
                        // tape is ready
                        if m == 0 {
                            self.tape[f].seek(-100);
                        } else {
                            self.tape[f].seek(m as i64);
                        }
                    }
                    8..=15 => {
                        while self.disk[f].busy() {
                            forward!(0, 1);
                        }
                        // disk is ready
                        if m != 0 {
                            panic!();
                        }
                        // TODO: rX に位置決めするとは？
                    }
                    16 => {
                        while self.card_reader.busy() {
                            forward!(0, 1);
                        }
                        // card reader is ready
                        // TODO: card reader に IOC はない？
                    }
                    17 => {
                        while self.card_punch.busy() {
                            forward!(0, 1);
                        }
                        // card punch is ready
                        // TODO: card punch に IOC はない？
                    }
                    18 => {
                        while self.line_printer.busy() {
                            forward!(0, 1);
                        }
                        // line printer is ready
                        if m != 0 {
                            panic!();
                        }
                        self.line_printer.next_page();
                    }
                    19 => {
                        while self.type_writer_terminal.busy() {
                            forward!(0, 1);
                        }
                        // type writer terminal is ready
                        // TODO: type writer terminal に IOC はない？
                    }
                    20 => {
                        while self.paper_tape.busy() {
                            forward!(0, 1);
                        }
                        // paper tape is ready
                        if m != 0 {
                            panic!();
                        }
                        self.paper_tape.seek(-100);
                    }
                    _ => {
                        unreachable!();
                    }
                }
                // TODO: clock実装
                forward!(1, 1);
            }
            36 => {
                // IN
                match f {
                    0..=7 => {
                        while self.tape[f].busy() {
                            forward!(0, 1);
                        }
                        // tape is ready
                        let v = self.tape[f].read();
                        for (i, x) in v.into_iter().enumerate() {
                            self.memory[(m + i as i64) as usize] = x.clone();
                        }
                    }
                    8..=15 => {
                        while self.disk[f].busy() {
                            forward!(0, 1);
                        }
                        // disk is ready
                        let v = self.disk[f].read();
                        for (i, x) in v.into_iter().enumerate() {
                            self.memory[(m + i as i64) as usize] = x.clone();
                        }
                    }
                    16 => {
                        while self.card_reader.busy() {
                            forward!(0, 1);
                        }
                        // card reader is ready
                        let v = self.card_reader.read();
                        for (i, x) in v.into_iter().enumerate() {
                            self.memory[(m + i as i64) as usize] = x.clone();
                        }
                    }
                    17 => {
                        while self.card_punch.busy() {
                            forward!(0, 1);
                        }
                        // card punch is ready
                        let v = self.card_punch.read();
                        for (i, x) in v.into_iter().enumerate() {
                            self.memory[(m + i as i64) as usize] = x.clone();
                        }
                    }
                    18 => {
                        while self.line_printer.busy() {
                            forward!(0, 1);
                        }
                        // line printer is ready
                        let v = self.line_printer.read();
                        for (i, x) in v.into_iter().enumerate() {
                            self.memory[(m + i as i64) as usize] = x.clone();
                        }
                    }
                    19 => {
                        while self.type_writer_terminal.busy() {
                            forward!(0, 1);
                        }
                        // type writer terminal is ready
                        let v = self.type_writer_terminal.read();
                        for (i, x) in v.into_iter().enumerate() {
                            self.memory[(m + i as i64) as usize] = x.clone();
                        }
                    }
                    20 => {
                        while self.paper_tape.busy() {
                            forward!(0, 1);
                        }
                        // paper tape is ready
                        let v = self.paper_tape.read();
                        for (i, x) in v.into_iter().enumerate() {
                            self.memory[(m + i as i64) as usize] = x.clone();
                        }
                    }
                    _ => {
                        unreachable!();
                    }
                }
                // TODO: clock実装
                forward!(1, 1);
            }
            37 => {
                // OUT
                match f {
                    0..=7 => {
                        while self.tape[f].busy() {
                            forward!(0, 1);
                        }
                        // tape is ready
                        let mut v = vec![];
                        for x in 0..iounit::Tape::block_size() {
                            v.push(&self.memory[(m + x as i64) as usize]);
                        }
                        self.tape[f].write(v);
                    }
                    8..=15 => {
                        while self.disk[f].busy() {
                            forward!(0, 1);
                        }
                        // disk is ready
                        let mut v = vec![];
                        for x in 0..iounit::Disk::block_size() {
                            v.push(&self.memory[(m + x as i64) as usize]);
                        }
                        self.disk[f].write(v);
                    }
                    16 => {
                        while self.card_reader.busy() {
                            forward!(0, 1);
                        }
                        // card reader is ready
                        let mut v = vec![];
                        for x in 0..iounit::CardReader::block_size() {
                            v.push(&self.memory[(m + x as i64) as usize]);
                        }
                        self.card_reader.write(v);
                    }
                    17 => {
                        while self.card_punch.busy() {
                            forward!(0, 1);
                        }
                        // card punch is ready
                        let mut v = vec![];
                        for x in 0..iounit::CardPunch::block_size() {
                            v.push(&self.memory[(m + x as i64) as usize]);
                        }
                        self.card_punch.write(v);
                    }
                    18 => {
                        while self.line_printer.busy() {
                            forward!(0, 1);
                        }
                        // line printer is ready
                        let mut v = vec![];
                        for x in 0..iounit::LinePrinter::block_size() {
                            v.push(&self.memory[(m + x as i64) as usize]);
                        }
                        self.line_printer.write(v);
                    }
                    19 => {
                        while self.type_writer_terminal.busy() {
                            forward!(0, 1);
                        }
                        // type writer terminal is ready
                        let mut v = vec![];
                        for x in 0..iounit::TypeWriterTerminal::block_size() {
                            v.push(&self.memory[(m + x as i64) as usize]);
                        }
                        self.type_writer_terminal.write(v);
                    }
                    20 => {
                        while self.paper_tape.busy() {
                            forward!(0, 1);
                        }
                        // paper tape is ready
                        let mut v = vec![];
                        for x in 0..iounit::PaperTape::block_size() {
                            v.push(&self.memory[(m + x as i64) as usize]);
                        }
                        self.paper_tape.write(v);
                    }
                    _ => {
                        unreachable!();
                    }
                }
                // TODO: clock実装
                forward!(1, 1);
            }
            38 => {
                // JRED
                match f {
                    0..=7 => {
                        if self.tape[f].ready() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    8..=15 => {
                        if self.disk[f].ready() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    16 => {
                        if self.card_reader.ready() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    17 => {
                        if self.card_punch.ready() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    18 => {
                        if self.line_printer.ready() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    19 => {
                        if self.type_writer_terminal.ready() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    20 => {
                        if self.paper_tape.ready() {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    _ => {
                        unreachable!();
                    }
                }
                forward!(0, 1);
            }
            39 => {
                // JMP/JSJ/JOV/JNOV/JL/JE/JG/JGE/JNE/JLE
                match f {
                    0 => {
                        // JMP
                        jump!();
                    }
                    1 => {
                        // JSJ
                        self.pc = m as usize;
                    }
                    2 => {
                        // JOV
                        if self.overflow {
                            self.overflow = false;
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    3 => {
                        // JNOV
                        if !self.overflow {
                            jump!();
                        } else {
                            self.overflow = false;
                            self.pc += 1;
                        }
                    }
                    4 => {
                        // JL
                        if self.comp == Ordering::Less {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    5 => {
                        // JE
                        if self.comp == Ordering::Equal {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    6 => {
                        // JG
                        if self.comp == Ordering::Greater {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    7 => {
                        // JGE
                        if self.comp != Ordering::Less {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    8 => {
                        // JNE
                        if self.comp != Ordering::Equal {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    9 => {
                        // JLE
                        if self.comp != Ordering::Greater {
                            jump!();
                        } else {
                            self.pc += 1;
                        }
                    }
                    _ => {
                        unreachable!();
                    }
                }
                forward!(0, 1);
            }
            40 => {
                macro40_47!(&mut self.reg_a);
                forward!(0, 1);
            }
            41 => {
                // J1+
                macro40_47!(&mut self.reg_i[0]);
                forward!(0, 1);
            }
            42 => {
                // J2+
                macro40_47!(&mut self.reg_i[1]);
                forward!(0, 1);
            }
            43 => {
                // J3+
                macro40_47!(&mut self.reg_i[2]);
                forward!(0, 1);
            }
            44 => {
                // J4+
                macro40_47!(&mut self.reg_i[3]);
                forward!(0, 1);
            }
            45 => {
                // J5+
                macro40_47!(&mut self.reg_i[4]);
                forward!(0, 1);
            }
            46 => {
                // J6+
                macro40_47!(&mut self.reg_i[5]);
                forward!(0, 1);
            }
            47 => {
                // JX+
                macro40_47!(&mut self.reg_x);
                forward!(0, 1);
            }
            48 => {
                // INCA/DECA/ENTA/ENNA
                macro48_55!(&mut self.reg_a);
                forward!(1, 1);
            }
            49 => {
                // INC1/DEC1/ENT1/ENN1
                macro48_55!(&mut self.reg_i[0]);
                forward!(1, 1);
            }
            50 => {
                // INC2/DEC2/ENT2/ENN2
                macro48_55!(&mut self.reg_i[1]);
                forward!(1, 1);
            }
            51 => {
                // INC3/DEC3/ENT3/ENN3
                macro48_55!(&mut self.reg_i[2]);
                forward!(1, 1);
            }
            52 => {
                // INC4/DEC4/ENT4/ENN4
                macro48_55!(&mut self.reg_i[3]);
                forward!(1, 1);
            }
            53 => {
                // INC5/DEC5/ENT5/ENN5
                macro48_55!(&mut self.reg_i[4]);
                forward!(1, 1);
            }
            54 => {
                // INC6/DEC6/ENT6/ENN6
                macro48_55!(&mut self.reg_i[5]);
                forward!(1, 1);
            }
            55 => {
                // INCX/DECX/ENTX/ENNX
                macro48_55!(&mut self.reg_x);
                forward!(1, 1);
            }
            56 => {
                // CMPA
                self.comp = instruction::comp(&self.reg_a, &self.memory[m as usize], f);
                forward!(1, 2);
            }
            57 => {
                // CMP1
                self.comp = instruction::comp(&self.reg_i[0], &self.memory[m as usize], f);
                forward!(1, 2);
            }
            58 => {
                // CMP2
                self.comp = instruction::comp(&self.reg_i[1], &self.memory[m as usize], f);
                forward!(1, 2);
            }
            59 => {
                // CMP3
                self.comp = instruction::comp(&self.reg_i[2], &self.memory[m as usize], f);
                forward!(1, 2);
            }
            60 => {
                // CMP4
                self.comp = instruction::comp(&self.reg_i[3], &self.memory[m as usize], f);
                forward!(1, 2);
            }
            61 => {
                // CMP5
                self.comp = instruction::comp(&self.reg_i[4], &self.memory[m as usize], f);
                forward!(1, 2);
            }
            62 => {
                // CMP6
                self.comp = instruction::comp(&self.reg_i[5], &self.memory[m as usize], f);
                forward!(1, 2);
            }
            63 => {
                // CMPX
                self.comp = instruction::comp(&self.reg_x, &self.memory[m as usize], f);
                forward!(1, 2);
            }
            _ => {
                unreachable!();
            }
        }

        Ok((current_pc, inst))
    }
}
