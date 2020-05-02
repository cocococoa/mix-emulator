use crate::mix_byte::Byte;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    Positive,
    Negative,
    NoSuchField,
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
            Sign::Positive | Sign::NoSuchField => sum,
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
                Sign::NoSuchField
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
        self.bytes.rotate_left(n)
    }
    pub fn rotate_right(&mut self, n: usize) {
        self.bytes.rotate_right(n)
    }

    // 以下は余計な気がする
    pub fn pair(l: usize, r: usize) -> usize {
        l * 8 + r
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
