use crate::vm::MixVM;

pub fn run(vm: &mut MixVM) -> Result<RunInfo, ()> {
    let mut info = RunInfo::new();
    let mut clock = 0;
    loop {
        match vm.step() {
            Ok((pc, _inst)) => {
                let current_clock = vm.clock();
                let clock_diff = (current_clock - clock) as usize;
                clock = current_clock;
                info.update(pc, clock_diff);
            }
            Err(()) => {
                // REACH HLT
                // TODO: get more info.
                break;
            }
        }
    }

    Ok(info)
}

#[derive(Debug)]
pub struct RunInfo {
    exec: Vec<usize>,
    clock: Vec<usize>,
}

impl RunInfo {
    pub fn new() -> Self {
        RunInfo {
            exec: vec![0; 4000],
            clock: vec![0; 4000],
        }
    }
    pub fn update(&mut self, address: usize, clock: usize) {
        self.exec[address] += 1;
        self.clock[address] += clock;
    }
    pub fn count_exec(&self, address: usize) -> usize {
        self.exec[address]
    }
    pub fn count_execs(&self, begin: usize, end: usize) -> Option<usize> {
        self.exec
            .get(begin..=end)
            .map(|slice| slice.iter().fold(0, |sum, item| sum + item))
    }
    pub fn count_clock(&self, address: usize) -> usize {
        self.clock[address]
    }
    pub fn count_clocks(&self, begin: usize, end: usize) -> Option<usize> {
        self.clock
            .get(begin..=end)
            .map(|slice| slice.iter().fold(0, |sum, item| sum + item))
    }
}
