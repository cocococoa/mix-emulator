use mix_emulator::asm::{debug_assemble, mix_format, release_assemble};
use mix_emulator::tools::run;
use mix_emulator::vm::{iounit, MixVM};
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "mix-emulator")]
struct Opt {
    // path to mix asm file
    #[structopt(name = "input file")]
    pub filepath: PathBuf,

    // run option
    #[structopt(short = "S", long = "preprocess", help = "preprocessor mode", conflicts_with_all(&["f", "c"]))]
    pub s: bool,
    #[structopt(short = "c", long = "compile", help = "compiler mode", conflicts_with_all(&["f", "S"]))]
    pub c: bool,
    #[structopt(short = "f", long = "format", help = "formatter mode", conflicts_with_all(&["S", "c"]))]
    pub f: bool,

    // format option
    #[structopt(short = "i", long = "inplace", help = "inplace edit, if specified")]
    pub i: bool,

    // verbose option
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,

    // iounit
    #[structopt(long = "tape0")]
    pub tape0: Option<PathBuf>,
    #[structopt(long = "tape1")]
    pub tape1: Option<PathBuf>,
    #[structopt(long = "tape2")]
    pub tape2: Option<PathBuf>,
    #[structopt(long = "tape3")]
    pub tape3: Option<PathBuf>,
    #[structopt(long = "tape4")]
    pub tape4: Option<PathBuf>,
    #[structopt(long = "tape5")]
    pub tape5: Option<PathBuf>,
    #[structopt(long = "tape6")]
    pub tape6: Option<PathBuf>,
    #[structopt(long = "tape7")]
    pub tape7: Option<PathBuf>,
    #[structopt(long = "disk0")]
    pub disk0: Option<PathBuf>,
    #[structopt(long = "disk1")]
    pub disk1: Option<PathBuf>,
    #[structopt(long = "disk2")]
    pub disk2: Option<PathBuf>,
    #[structopt(long = "disk3")]
    pub disk3: Option<PathBuf>,
    #[structopt(long = "disk4")]
    pub disk4: Option<PathBuf>,
    #[structopt(long = "disk5")]
    pub disk5: Option<PathBuf>,
    #[structopt(long = "disk6")]
    pub disk6: Option<PathBuf>,
    #[structopt(long = "disk7")]
    pub disk7: Option<PathBuf>,
    #[structopt(long = "cardreader")]
    pub card_reader: Option<PathBuf>,
    #[structopt(long = "cardpunch")]
    pub card_punch: Option<PathBuf>,
    #[structopt(long = "lineprinter")]
    pub line_printer: Option<PathBuf>,
    #[structopt(long = "twt")]
    pub type_writer_terminal: Option<PathBuf>,
    #[structopt(long = "papertape")]
    pub paper_tape: Option<PathBuf>,
}

pub fn compile(src_code: String, path: &PathBuf) -> std::io::Result<()> {
    let (entry_point, binary) = release_assemble(&src_code);
    let mut file = fs::File::create(path)?;
    writeln!(file, "{:<04}", entry_point)?;
    for (addr, word) in binary {
        writeln!(file, "{:<04} {}", addr, word)?;
    }

    Ok(())
}

pub fn format(src_code: String, inplace: bool, path: &PathBuf) -> std::io::Result<()> {
    let formatted = mix_format(&src_code);
    if inplace {
        // write formatted code into the opt.filepath
        let mut file = fs::File::create(path)?;
        write!(file, "{}", formatted)?;
        file.flush()?;

        Ok(())
    } else {
        // output to stdout
        println!("{}", formatted);

        Ok(())
    }
}

fn set_iounit(vm: &mut MixVM, opt: &Opt) -> std::io::Result<()> {
    if opt.tape0.is_some() {}
    if opt.tape1.is_some() {
        let input = fs::read_to_string(opt.tape1.as_ref().unwrap())?;
        for i in input.lines() {
            let i = i.trim();
            let mut v = vec![];
            for j in i.split(",") {
                v.push(j.trim().parse::<i64>().unwrap());
            }
            loop {
                v.push(0);
                if v.len() == 100 {
                    break;
                }
            }
            vm.read_binary(1, v);
        }
    }
    if opt.tape2.is_some() {}
    if opt.tape3.is_some() {}
    if opt.tape4.is_some() {}
    if opt.tape5.is_some() {}
    if opt.tape6.is_some() {}
    if opt.tape7.is_some() {}
    if opt.disk0.is_some() {}
    if opt.disk1.is_some() {}
    if opt.disk2.is_some() {}
    if opt.disk3.is_some() {}
    if opt.disk4.is_some() {}
    if opt.disk5.is_some() {}
    if opt.disk6.is_some() {}
    if opt.disk7.is_some() {}
    if opt.card_reader.is_some() {
        let input = fs::read_to_string(opt.card_reader.as_ref().unwrap())?;
        for i in input.lines() {
            if i.len() > 0 && i.len() % 5 == 0 {
                let mut v = vec![];
                for j in 0..(i.len() / 5) {
                    v.push(i.get((5 * j)..(5 * j + 5)).unwrap().to_string());
                }
                vm.read(16, v);
            }
        }
    }
    if opt.card_punch.is_some() {}
    if opt.line_printer.is_some() {}
    if opt.type_writer_terminal.is_some() {}
    if opt.paper_tape.is_some() {}

    Ok(())
}

fn main() {
    let opt = Opt::from_args();

    // 1. check suffix
    if &opt.filepath.extension().map(|p| p.to_str()).flatten() != &Some("mix")
        && &opt.filepath.extension().map(|p| p.to_str()).flatten() != &Some("mixbin")
    {
        println!(
            "file format not recognized\nfile extension should be .mix: {}",
            opt.filepath.to_str().unwrap()
        );
        return;
    }

    // 2. open file
    let src_code = fs::read_to_string(&opt.filepath);
    if src_code.is_err() {
        println!("No such file: {}", opt.filepath.to_str().unwrap());
        return;
    }
    let src_code = src_code.unwrap();

    // 3. main
    if opt.s {
        // preprocess
    } else if opt.c {
        // compile
        let res = compile(src_code, &opt.filepath.with_extension("mixbin"));
        if res.is_err() {
            println!("{}", res.unwrap_err().to_string());
        }
    } else if opt.f {
        // format
        let res = format(src_code, opt.i, &opt.filepath);
        if res.is_err() {
            println!("{}", res.unwrap_err().to_string());
        }
    } else {
        // run
        let src_code = mix_format(&src_code);
        let (entry_point, binary, table) = debug_assemble(&src_code);
        let mut vm = MixVM::new();
        vm.load(&binary);
        vm.set_pc(entry_point);

        // set iounit
        let _ = set_iounit(&mut vm, &opt);

        // release
        let run_info = run(&mut vm).unwrap();
        if opt.verbose {
            // verbose
            // 10 + 5 + 15 + 10 + 10
            println!("LOC       OPE  ADDRESS             COUNT     CLOCK");
            for (i, line) in src_code.lines().enumerate() {
                let address = table.get(&i);
                if address.is_none() {
                    continue;
                }
                let address = *address.unwrap();
                let count = run_info.count_exec(address);
                let clock = run_info.count_clock(address);
                println!("{:30}{:10}{:10}", line, count, clock);
            }
        }
        for i in 0..21 {
            let s = if i >= 16 {
                vm.print(i)
            } else {
                vm.print_binary(i)
                    .iter()
                    .map(|int| (*int).to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            if s.len() > 0 {
                println!("[{}]\n{}", iounit(i), s);
            }
        }
    }
}
