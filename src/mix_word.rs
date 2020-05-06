static mut MAX_BYTE: u32 = 64;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Byte {
    v: u32,
}

impl Byte {
    pub fn new(v: u32) -> Self {
        Byte { v: v }
    }
    pub fn val(&self) -> u32 {
        self.v
    }
    pub fn val_mut(&mut self) -> &mut u32 {
        &mut self.v
    }
    pub fn inc(&mut self) -> bool {
        self.v += 1;
        if self.v >= Byte::max() {
            self.v = 0;
            true
        } else {
            false
        }
    }

    pub fn max() -> u32 {
        unsafe { MAX_BYTE }
    }
    pub fn set_max(max: u32) -> Result<(), ()> {
        if max < 64 || 100 < max {
            Err(())
        } else {
            unsafe {
                MAX_BYTE = max;
            }
            Ok(())
        }
    }
    pub fn word_max() -> i64 {
        let byte_max = Byte::max() as i64;
        byte_max.pow(5)
    }
}

impl std::fmt::Display for Byte {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte() {
        let mut b: Byte = Default::default();
        assert_eq!(Byte::new(0), b);
        assert_eq!(0, b.val());
        *b.val_mut() = 63;
        assert_eq!(63, b.val());
        assert!(b.inc());
        assert_eq!(Byte::new(0), b);
    }
}

pub type Register = WordImpl;
pub type Memory = Vec<WordImpl>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WordImpl {
    sign: Sign,
    bytes: Vec<Byte>,
}

impl WordImpl {
    pub fn word() -> Self {
        WordImpl {
            sign: Sign::Positive,
            bytes: vec![Byte::default(); 5],
        }
    }
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    pub fn val(&self) -> i64 {
        let mut sum = 0;
        for x in &self.bytes {
            sum *= Byte::max() as i64;
            sum += x.val() as i64;
        }

        match self.sign {
            Sign::Positive => sum,
            Sign::Negative => -sum,
        }
    }
    pub fn from_val(mut val: i64) -> Self {
        let mut ret = WordImpl::word();
        if val < 0 {
            ret.sign = Sign::Negative;
            val *= -1;
        }
        for x in (0..5).rev() {
            ret.bytes[x] = Byte::new((val % Byte::max() as i64) as u32);
            val /= Byte::max() as i64;
        }
        if val != 0 {
            // unimplemented
            panic!();
        }

        ret
    }
    pub fn from_seq(s: Sign, v: &Vec<u32>) -> Self {
        WordImpl {
            sign: s,
            bytes: v.iter().map(|byte| Byte::new(*byte)).collect::<Vec<_>>(),
        }
    }
    pub fn sign(&self) -> &Sign {
        &self.sign
    }
    pub fn sign_mut(&mut self) -> &mut Sign {
        &mut self.sign
    }
    pub fn byte(&self, pos: usize) -> Option<&Byte> {
        if pos >= self.len() {
            None
        } else {
            Some(&self.bytes[pos])
        }
    }
    pub fn byte_mut(&mut self, pos: usize) -> Option<&mut Byte> {
        if pos >= self.len() {
            None
        } else {
            Some(&mut self.bytes[pos])
        }
    }
    pub fn subword(&self, fspec: usize) -> Option<Self> {
        let mut l = fspec / 8;
        let r = fspec % 8;
        if l <= r && r <= self.len() {
            let sign = if l == 0 {
                l += 1;
                self.sign
            } else {
                Sign::Positive
            };
            let mut bytes: Vec<Byte> = vec![];
            for x in l..=r {
                bytes.push(self.bytes[x - 1]);
            }

            Some(WordImpl {
                sign: sign,
                bytes: bytes,
            })
        } else {
            None
        }
    }
    pub fn inc(&mut self) {
        *self = WordImpl::from_val(self.val() + 1);
    }
    pub fn shift_left(&mut self, n: usize) {
        for i in 0..(self.len() - n) {
            self.bytes[i] = self.bytes[i + n];
        }
        for i in (self.len() - n)..self.len() {
            self.bytes[i] = Byte::default();
        }
    }
    pub fn shift_right(&mut self, n: usize) {
        for i in (n..self.len()).rev() {
            self.bytes[i] = self.bytes[i - n];
        }
        for i in 0..n {
            self.bytes[i] = Byte::default();
        }
    }
    pub fn rotate_left(&mut self, n: usize) {
        let len = self.len();
        self.bytes.rotate_left(n % len)
    }
    pub fn rotate_right(&mut self, n: usize) {
        let len = self.len();
        self.bytes.rotate_right(n % len)
    }

    // 以下は余計な気がする
    pub fn pair(l: usize, r: usize) -> usize {
        l * 8 + r
    }
    pub fn unpair(fspec: usize) -> (usize, usize) {
        let l = fspec / 8;
        let r = fspec % 8;
        (l, r)
    }
    pub fn address(&self) -> i64 {
        self.subword(WordImpl::pair(0, 2)).unwrap().val()
    }
    pub fn index(&self) -> i64 {
        self.subword(WordImpl::pair(3, 3)).unwrap().val()
    }
    pub fn modification(&self) -> i64 {
        self.subword(WordImpl::pair(4, 4)).unwrap().val()
    }
    pub fn operation(&self) -> i64 {
        self.subword(WordImpl::pair(5, 5)).unwrap().val()
    }
}

impl std::fmt::Display for WordImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self.sign {
            Sign::Positive => "+",
            Sign::Negative => "-",
        };
        let b = self
            .bytes
            .iter()
            .map(|byte| format!("{} ", byte))
            .collect::<String>();
        write!(f, "{} {}", s, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word() {
        let mut reg = WordImpl::word();
        assert_eq!(0, reg.val());
        reg.inc();
        assert_eq!(1, reg.val());
        let mut reg = WordImpl::from_val(63);
        assert_eq!(63, reg.val());
        reg.inc();
        assert_eq!(64, reg.val());
        assert_eq!(0, reg.operation());
        assert_eq!(1, reg.modification());
        let mut reg = WordImpl::from_val(10);
        assert_eq!(10, reg.val());
        reg.shift_left(1);
        assert_eq!(10 * Byte::max() as i64, reg.val());
        reg.shift_left(1);
        assert_eq!(10 * (Byte::max() as i64) * (Byte::max() as i64), reg.val());
        reg.shift_right(1);
        assert_eq!(10 * Byte::max() as i64, reg.val());
        reg.shift_right(1);
        assert_eq!(10, reg.val());

        assert_eq!(10, WordImpl::from_val(10).val());
        assert_eq!(1_000, WordImpl::from_val(1_000).val());
        assert_eq!(10_000_000, WordImpl::from_val(10_000_000).val());
        assert_eq!(1_000_000_000, WordImpl::from_val(1_000_000_000).val());
    }
}
