use crate::common::CHAR_TABLE;
use crate::mix_word::WordImpl;

pub trait IOUnit {
    type Word;
    fn block_size() -> usize;

    fn read(&self) -> Vec<&Self::Word>;
    fn write(&mut self, w: Vec<&Self::Word>);
    fn seek(&mut self, offset: i64);

    fn ready(&self) -> bool;
    fn busy(&self) -> bool;

    fn print(&self) -> String;
}

#[derive(Debug, Default, Clone)]
pub struct Tape {
    data: Vec<WordImpl>,
    pos: usize,
}
#[derive(Debug, Default, Clone)]
pub struct Disk {
    data: Vec<WordImpl>,
    pos: usize,
}
#[derive(Debug, Default)]
pub struct CardReader {
    data: Vec<WordImpl>,
    pos: usize,
}
#[derive(Debug, Default)]
pub struct CardPunch {
    data: Vec<WordImpl>,
    pos: usize,
}
#[derive(Debug, Default)]
pub struct LinePrinter {
    data: Vec<WordImpl>,
    page: Vec<usize>,
    pos: usize,
}
#[derive(Debug, Default)]
pub struct TypeWriterTerminal {
    data: Vec<WordImpl>,
    pos: usize,
}
#[derive(Debug, Default)]
pub struct PaperTape {
    data: Vec<WordImpl>,
    pos: usize,
}

fn num_to_char(n: usize) -> char {
    CHAR_TABLE[n]
}

macro_rules! impl_io_trait {
    ($machine: ty, $block_size: expr) => {
        impl IOUnit for $machine {
            type Word = WordImpl;
            fn block_size() -> usize {
                $block_size
            }
            fn read(&self) -> Vec<&Self::Word> {
                let mut ret = vec![];
                for i in 0..(Self::block_size()) {
                    ret.push(&self.data[self.pos + i]);
                }
                ret
            }
            fn write(&mut self, w: Vec<&Self::Word>) {
                // resize
                if self.data.len() < self.pos + Self::block_size() {
                    self.data
                        .resize(self.pos + Self::block_size(), Self::Word::from_val(0));
                }
                for i in 0..(Self::block_size()) {
                    self.data[self.pos + i] = (*w[i]).clone();
                }
                self.pos += Self::block_size();
            }
            fn seek(&mut self, offset: i64) {
                let offset = offset * Self::block_size() as i64;
                self.pos = (if (self.pos as i64) + offset >= 0 {
                    self.pos as i64 + offset
                } else {
                    0
                }) as usize;
            }
            fn ready(&self) -> bool {
                true
            }
            fn busy(&self) -> bool {
                false
            }
            fn print(&self) -> String {
                let mut ret = "".to_string();
                for (line, x) in self.data.iter().enumerate() {
                    for i in 0..5 {
                        let num = x.byte(i).unwrap().val();
                        ret.push(num_to_char(num as usize));
                    }
                    if (line + 1) % Self::block_size() == 0 {
                        ret.push('\n');
                    }
                }
                ret
            }
        }
    };
}

impl_io_trait!(Tape, 100);
impl_io_trait!(Disk, 100);
impl_io_trait!(CardReader, 16);
impl_io_trait!(CardPunch, 16);
impl_io_trait!(LinePrinter, 24);
impl_io_trait!(TypeWriterTerminal, 14);
impl_io_trait!(PaperTape, 14);

impl LinePrinter {
    pub fn next_page(&mut self) {
        self.page.push(self.pos);
    }
}
