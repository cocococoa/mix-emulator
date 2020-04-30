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

        assert_eq!(64, Byte::max());
        assert_eq!(Err(()), Byte::set_max(1000));
        assert_eq!(Ok(()), Byte::set_max(100));
        assert_eq!(100, Byte::max());
        *b.val_mut() = 99;
        assert!(b.inc());
    }
}
