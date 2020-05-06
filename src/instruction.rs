use crate::mix_word::{Byte, Memory, Register, Sign};
use std::cmp::Ordering;

// ロード命令
pub fn load(from: &Register, to: &mut Register, fspec: usize) {
    let cp = from.subword(fspec).unwrap();
    *to.sign_mut() = *cp.sign();
    for x in 0..to.len() {
        *to.byte_mut(x).unwrap() = Byte::default();
    }
    for x in 0..cp.len() {
        *to.byte_mut(x + to.len() - cp.len()).unwrap() = *cp.byte(x).unwrap();
    }
}
pub fn loadn(from: &Register, to: &mut Register, fspec: usize) {
    let cp = from.subword(fspec).unwrap();
    *to.sign_mut() = match *cp.sign() {
        Sign::Positive => Sign::Negative,
        Sign::Negative => Sign::Positive,
    };
    for x in 0..to.len() {
        *to.byte_mut(x).unwrap() = Byte::default();
    }
    for x in 0..cp.len() {
        *to.byte_mut(x + to.len() - cp.len()).unwrap() = *cp.byte(x).unwrap();
    }
}

// ストア命令
pub fn store(from: &Register, to: &mut Register, fspec: usize) {
    let (mut l, r) = Register::unpair(fspec);
    if l == 0 {
        *to.sign_mut() = *from.sign();
        l += 1;
    }
    for i in l..=r {
        *to.byte_mut(i - 1).unwrap() = *from.byte(i + 4 - r).unwrap();
    }
}
pub fn store_zero(to: &mut Register) {
    for x in 0..to.len() {
        *to.byte_mut(x).unwrap() = Byte::default();
    }
}

// 数値演算命令
pub fn add(from: &Register, to: &mut Register, fspec: usize, overflow: &mut bool) {
    let mut val = from.subword(fspec).unwrap().val() + to.val();
    let sign = *to.sign();
    if val.abs() >= Byte::word_max() {
        *overflow = true;
        val %= Byte::word_max();
    }
    *to = Register::from_val(val);
    if val == 0 {
        *to.sign_mut() = sign;
    }
}
pub fn sub(from: &Register, to: &mut Register, fspec: usize, overflow: &mut bool) {
    let mut val = -from.subword(fspec).unwrap().val() + to.val();
    let sign = *to.sign();
    if val.abs() >= Byte::word_max() {
        *overflow = true;
        val %= Byte::word_max();
    }
    *to = Register::from_val(val);
    if val == 0 {
        *to.sign_mut() = sign;
    }
}
pub fn mul(from: &Register, to_a: &mut Register, to_x: &mut Register, fspec: usize) {
    let from = from.subword(fspec).unwrap();
    let val = from.val() * to_a.val();
    let val = val.abs();
    let sign = if from.sign() != to_a.sign() {
        Sign::Negative
    } else {
        Sign::Positive
    };
    *to_a = Register::from_val(val / Byte::word_max());
    *to_x = Register::from_val(val % Byte::word_max());
    *to_a.sign_mut() = sign;
    *to_x.sign_mut() = sign;
}
pub fn div(
    from: &Register,
    to_a: &mut Register,
    to_x: &mut Register,
    fspec: usize,
    overflow: &mut bool,
) {
    let from = from.subword(fspec).unwrap();
    let divisor = from.subword(fspec).unwrap().val();
    let divisor = divisor.abs();
    if divisor == 0 || to_a.val().abs() >= divisor {
        *overflow = true;
        return;
    }
    let rax = to_a.val() * Byte::word_max() + to_x.val();
    let rax = rax.abs();
    let sign_a = if from.sign() != to_a.sign() {
        Sign::Negative
    } else {
        Sign::Positive
    };
    let sign_x = *to_a.sign();

    let div = rax / divisor;
    let remainder = rax % divisor;
    *to_a = Register::from_val(div);
    *to_x = Register::from_val(remainder);
    *to_a.sign_mut() = sign_a;
    *to_x.sign_mut() = sign_x;
}

// アドレス転送命令
pub fn ent(m: i64, sign: &Sign, to: &mut Register) {
    *to = Register::from_val(m);
    if m == 0 {
        *to.sign_mut() = *sign;
    }
}
pub fn entn(m: i64, sign: &Sign, to: &mut Register) {
    *to = Register::from_val(-m);
    if m == 0 {
        *to.sign_mut() = *sign;
    }
}
pub fn inc(m: i64, to: &mut Register, overflow: &mut bool) {
    let tmp_reg = Register::from_val(m);
    add(&tmp_reg, to, Register::pair(0, 5), overflow);
}
pub fn inci(m: i64, to: &mut Register) {
    let tmp_reg = Register::from_val(m);
    let mut overflow = false;
    add(&tmp_reg, to, Register::pair(0, 5), &mut overflow);
    if overflow {
        panic!();
    }
}
pub fn dec(m: i64, to: &mut Register, overflow: &mut bool) {
    let tmp_reg = Register::from_val(m);
    sub(&tmp_reg, to, Register::pair(0, 5), overflow);
}
pub fn deci(m: i64, to: &mut Register) {
    let tmp_reg = Register::from_val(m);
    let mut overflow = false;
    sub(&tmp_reg, to, Register::pair(0, 5), &mut overflow);
    if overflow {
        panic!();
    }
}

// 比較命令
pub fn comp(lhs: &Register, rhs: &Register, fspec: usize) -> Ordering {
    lhs.subword(fspec)
        .unwrap()
        .val()
        .cmp(&rhs.subword(fspec).unwrap().val())
}

// ジャンプ命令
// pub fn jmp()

// 種々の命令
pub fn shift_left(n: i64, reg: &mut Register) {
    reg.shift_left(n as usize);
}
pub fn shift_right(n: i64, reg: &mut Register) {
    reg.shift_right(n as usize);
}
pub fn shift_left_pair(n: i64, reg_a: &mut Register, reg_x: &mut Register) {
    let mut reg = Register::from_seq(
        Sign::Negative,
        &vec![
            reg_a.byte(0).unwrap().val(),
            reg_a.byte(1).unwrap().val(),
            reg_a.byte(2).unwrap().val(),
            reg_a.byte(3).unwrap().val(),
            reg_a.byte(4).unwrap().val(),
            reg_x.byte(0).unwrap().val(),
            reg_x.byte(1).unwrap().val(),
            reg_x.byte(2).unwrap().val(),
            reg_x.byte(3).unwrap().val(),
            reg_x.byte(4).unwrap().val(),
        ],
    );
    reg.shift_left(n as usize);
    for i in 0..5 {
        *reg_a.byte_mut(i).unwrap() = *reg.byte(i).unwrap();
        *reg_x.byte_mut(i).unwrap() = *reg.byte(5 + i).unwrap();
    }
}
pub fn shift_right_pair(n: i64, reg_a: &mut Register, reg_x: &mut Register) {
    let mut reg = Register::from_seq(
        Sign::Negative,
        &vec![
            reg_a.byte(0).unwrap().val(),
            reg_a.byte(1).unwrap().val(),
            reg_a.byte(2).unwrap().val(),
            reg_a.byte(3).unwrap().val(),
            reg_a.byte(4).unwrap().val(),
            reg_x.byte(0).unwrap().val(),
            reg_x.byte(1).unwrap().val(),
            reg_x.byte(2).unwrap().val(),
            reg_x.byte(3).unwrap().val(),
            reg_x.byte(4).unwrap().val(),
        ],
    );
    reg.shift_right(n as usize);
    for i in 0..5 {
        *reg_a.byte_mut(i).unwrap() = *reg.byte(i).unwrap();
        *reg_x.byte_mut(i).unwrap() = *reg.byte(5 + i).unwrap();
    }
}
pub fn rotate_left_pair(n: i64, reg_a: &mut Register, reg_x: &mut Register) {
    let mut reg = Register::from_seq(
        Sign::Negative,
        &vec![
            reg_a.byte(0).unwrap().val(),
            reg_a.byte(1).unwrap().val(),
            reg_a.byte(2).unwrap().val(),
            reg_a.byte(3).unwrap().val(),
            reg_a.byte(4).unwrap().val(),
            reg_x.byte(0).unwrap().val(),
            reg_x.byte(1).unwrap().val(),
            reg_x.byte(2).unwrap().val(),
            reg_x.byte(3).unwrap().val(),
            reg_x.byte(4).unwrap().val(),
        ],
    );
    reg.rotate_left(n as usize);
    for i in 0..5 {
        *reg_a.byte_mut(i).unwrap() = *reg.byte(i).unwrap();
        *reg_x.byte_mut(i).unwrap() = *reg.byte(5 + i).unwrap();
    }
}
pub fn rotate_right_pair(n: i64, reg_a: &mut Register, reg_x: &mut Register) {
    let mut reg = Register::from_seq(
        Sign::Negative,
        &vec![
            reg_a.byte(0).unwrap().val(),
            reg_a.byte(1).unwrap().val(),
            reg_a.byte(2).unwrap().val(),
            reg_a.byte(3).unwrap().val(),
            reg_a.byte(4).unwrap().val(),
            reg_x.byte(0).unwrap().val(),
            reg_x.byte(1).unwrap().val(),
            reg_x.byte(2).unwrap().val(),
            reg_x.byte(3).unwrap().val(),
            reg_x.byte(4).unwrap().val(),
        ],
    );
    reg.rotate_right(n as usize);
    for i in 0..5 {
        *reg_a.byte_mut(i).unwrap() = *reg.byte(i).unwrap();
        *reg_x.byte_mut(i).unwrap() = *reg.byte(5 + i).unwrap();
    }
}
pub fn mov(from: i64, to: i64, n: i64, mem: &mut Memory) {
    for x in 0..n {
        mem[(to + x) as usize] = mem[(from + x) as usize].clone();
    }
}

// 入出力装置

// 変換命令
pub fn to_num(reg_a: &mut Register, reg_x: &Register, overflow: &mut bool) {
    let mut sum: i64 = (reg_a.byte(0).unwrap().val() % 10) as i64;
    sum = 10 * sum + (reg_a.byte(1).unwrap().val() % 10) as i64;
    sum = 10 * sum + (reg_a.byte(2).unwrap().val() % 10) as i64;
    sum = 10 * sum + (reg_a.byte(3).unwrap().val() % 10) as i64;
    sum = 10 * sum + (reg_a.byte(4).unwrap().val() % 10) as i64;
    sum = 10 * sum + (reg_x.byte(0).unwrap().val() % 10) as i64;
    sum = 10 * sum + (reg_x.byte(1).unwrap().val() % 10) as i64;
    sum = 10 * sum + (reg_x.byte(2).unwrap().val() % 10) as i64;
    sum = 10 * sum + (reg_x.byte(3).unwrap().val() % 10) as i64;
    sum = 10 * sum + (reg_x.byte(4).unwrap().val() % 10) as i64;
    let sign = *reg_a.sign();
    if sum.abs() > Byte::word_max() {
        sum &= Byte::word_max();
        *overflow = true;
    }
    *reg_a = Register::from_val(sum % Byte::word_max());
    *reg_a.sign_mut() = sign;
}
pub fn to_char(reg_a: &mut Register, reg_x: &mut Register) {
    let mut val = reg_a.val().abs();
    *reg_x.byte_mut(4).unwrap() = Byte::new(30 + (val % 10) as u32);
    val /= 10;
    *reg_x.byte_mut(3).unwrap() = Byte::new(30 + (val % 10) as u32);
    val /= 10;
    *reg_x.byte_mut(2).unwrap() = Byte::new(30 + (val % 10) as u32);
    val /= 10;
    *reg_x.byte_mut(1).unwrap() = Byte::new(30 + (val % 10) as u32);
    val /= 10;
    *reg_x.byte_mut(0).unwrap() = Byte::new(30 + (val % 10) as u32);
    val /= 10;
    *reg_a.byte_mut(4).unwrap() = Byte::new(30 + (val % 10) as u32);
    val /= 10;
    *reg_a.byte_mut(3).unwrap() = Byte::new(30 + (val % 10) as u32);
    val /= 10;
    *reg_a.byte_mut(2).unwrap() = Byte::new((30 + val % 10) as u32);
    val /= 10;
    *reg_a.byte_mut(1).unwrap() = Byte::new(30 + (val % 10) as u32);
    val /= 10;
    *reg_a.byte_mut(0).unwrap() = Byte::new(30 + (val % 10) as u32);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        macro_rules! test {
            ($fspec: expr, $sign: expr, $v: expr) => {
                let mut reg_a = Register::from_seq(Sign::Negative, &vec![1, 2, 3, 4, 5]);
                let from = Register::from_seq(Sign::Negative, &vec![1, 16, 3, 4, 5]);
                load(&from, &mut reg_a, $fspec);
                assert_eq!(Register::from_seq($sign, &$v), reg_a);
            };
        }

        test!(Register::pair(0, 5), Sign::Negative, vec![1, 16, 3, 4, 5]);
        test!(Register::pair(1, 5), Sign::Positive, vec![1, 16, 3, 4, 5]);
        test!(Register::pair(3, 5), Sign::Positive, vec![0, 0, 3, 4, 5]);
        test!(Register::pair(0, 3), Sign::Negative, vec![0, 0, 1, 16, 3]);
        test!(Register::pair(4, 4), Sign::Positive, vec![0, 0, 0, 0, 4]);
        test!(Register::pair(0, 0), Sign::Negative, vec![0, 0, 0, 0, 0]);
        test!(Register::pair(1, 1), Sign::Positive, vec![0, 0, 0, 0, 1]);
    }
    #[test]
    fn test_store() {
        macro_rules! test {
            ($fspec: expr, $sign: expr, $v: expr) => {
                let reg_a = Register::from_seq(Sign::Positive, &vec![6, 7, 8, 9, 0]);
                let mut to = Register::from_seq(Sign::Negative, &vec![1, 2, 3, 4, 5]);
                store(&reg_a, &mut to, $fspec);
                assert_eq!(Register::from_seq($sign, &$v), to);
            };
        }

        test!(Register::pair(0, 5), Sign::Positive, vec![6, 7, 8, 9, 0]);
        test!(Register::pair(1, 5), Sign::Negative, vec![6, 7, 8, 9, 0]);
        test!(Register::pair(5, 5), Sign::Negative, vec![1, 2, 3, 4, 0]);
        test!(Register::pair(2, 2), Sign::Negative, vec![1, 0, 3, 4, 5]);
        test!(Register::pair(2, 3), Sign::Negative, vec![1, 9, 0, 4, 5]);
        test!(Register::pair(0, 1), Sign::Positive, vec![0, 2, 3, 4, 5]);
    }
    #[test]
    fn test_add() {
        macro_rules! test {
            ($v: expr) => {
                let mut reg_a = Register::from_seq(Sign::Positive, &$v);
                let mut mem2000 = Register::from_val(0);
                store(&reg_a, &mut mem2000, Register::pair(0, 5));
                load(&mem2000, &mut reg_a, Register::pair(5, 5));
                add(&mem2000, &mut reg_a, Register::pair(4, 4), &mut false);
                add(&mem2000, &mut reg_a, Register::pair(3, 3), &mut false);
                add(&mem2000, &mut reg_a, Register::pair(2, 2), &mut false);
                add(&mem2000, &mut reg_a, Register::pair(1, 1), &mut false);
                assert_eq!($v.iter().fold(0, |sum, i| sum + i), reg_a.val());

                let mut reg_a = Register::from_seq(Sign::Negative, &$v);
                let mut mem2000 = Register::from_val(0);
                store(&reg_a, &mut mem2000, Register::pair(0, 5));
                load(&mem2000, &mut reg_a, Register::pair(5, 5));
                add(&mem2000, &mut reg_a, Register::pair(4, 4), &mut false);
                add(&mem2000, &mut reg_a, Register::pair(3, 3), &mut false);
                add(&mem2000, &mut reg_a, Register::pair(2, 2), &mut false);
                add(&mem2000, &mut reg_a, Register::pair(1, 1), &mut false);
                assert_eq!($v.iter().fold(0, |sum, i| sum + i), reg_a.val());
            };
        }

        test!(vec![1, 2, 3, 4, 5]);
        test!(vec![61, 62, 63, 59, 58]);
        test!(vec![10, 0, 3, 0, 5]);
        test!(vec![0, 0, 0, 0, 0]);
    }
    #[test]
    fn test_arithmetics() {
        /*** sample1 ***/
        let mut reg_a = Register::from_seq(Sign::Positive, &vec![19, 18, 1, 2, 22]);
        let mem1000 = Register::from_seq(Sign::Positive, &vec![1, 36, 5, 0, 50]);
        add(&mem1000, &mut reg_a, Register::pair(0, 5), &mut false);
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![20, 54, 6, 3, 8]),
            reg_a
        );
        /*** sample2 ***/
        let mut reg_a = Register::from_seq(Sign::Negative, &vec![19, 18, 0, 0, 9]);
        let mem1000 = Register::from_seq(Sign::Negative, &vec![31, 16, 2, 22, 0]);
        sub(&mem1000, &mut reg_a, Register::pair(0, 5), &mut false);
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![11, 62, 2, 21, 55]),
            reg_a
        );
        /*** sample3 ***/
        let mut reg_a = Register::from_seq(Sign::Positive, &vec![1, 1, 1, 1, 1]);
        let mut reg_x = Register::from_seq(Sign::Negative, &vec![19, 18, 0, 0, 9]);
        let mem1000 = Register::from_seq(Sign::Positive, &vec![1, 1, 1, 1, 1]);
        mul(&mem1000, &mut reg_a, &mut reg_x, Register::pair(0, 5));
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![0, 1, 2, 3, 4]),
            reg_a
        );
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![5, 4, 3, 2, 1]),
            reg_x
        );
        /*** sample4 ***/
        let mut reg_a = Register::from_val(-112);
        let mut reg_x = Register::from_seq(Sign::Negative, &vec![19, 18, 0, 0, 9]);
        let mem1000 = Register::from_seq(Sign::Positive, &vec![2, 0, 0, 0, 0]);
        mul(&mem1000, &mut reg_a, &mut reg_x, Register::pair(1, 1));
        assert_eq!(
            Register::from_seq(Sign::Negative, &vec![0, 0, 0, 0, 0]),
            reg_a
        );
        assert_eq!(
            Register::from_seq(Sign::Negative, &vec![0, 0, 0, 3, 32]),
            reg_x
        );
        /*** sample5 ***/
        let mut reg_a = Register::from_seq(Sign::Negative, &vec![50, 0, 1, 48, 4]);
        let mut reg_x = Register::from_seq(Sign::Negative, &vec![19, 18, 0, 0, 9]);
        let mem1000 = Register::from_seq(Sign::Negative, &vec![2, 0, 0, 0, 0]);
        mul(&mem1000, &mut reg_a, &mut reg_x, Register::pair(0, 5));
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![1, 36, 0, 3, 32]),
            reg_a
        );
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![8, 0, 0, 0, 0]),
            reg_x
        );
        /*** sample6 ***/
        let mut reg_a = Register::from_seq(Sign::Positive, &vec![0, 0, 0, 0, 0]);
        let mut reg_x = Register::from_seq(Sign::Negative, &vec![0, 0, 0, 0, 17]);
        let mem1000 = Register::from_seq(Sign::Positive, &vec![0, 0, 0, 0, 3]);
        div(
            &mem1000,
            &mut reg_a,
            &mut reg_x,
            Register::pair(0, 5),
            &mut false,
        );
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![0, 0, 0, 0, 5]),
            reg_a
        );
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![0, 0, 0, 0, 2]),
            reg_x
        );
        /*** sample7 ***/
        let mut reg_a = Register::from_seq(Sign::Negative, &vec![0, 0, 0, 0, 0]);
        let mut reg_x = Register::from_seq(Sign::Positive, &vec![19, 19, 0, 3, 1]);
        let mem1000 = Register::from_seq(Sign::Negative, &vec![0, 0, 0, 2, 0]);
        div(
            &mem1000,
            &mut reg_a,
            &mut reg_x,
            Register::pair(0, 5),
            &mut false,
        );
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![0, 9, 41, 32, 1]),
            reg_a
        );
        assert_eq!(
            Register::from_seq(Sign::Negative, &vec![0, 0, 0, 1, 1]),
            reg_x
        );
    }
    #[test]
    fn test_ent() {
        let mut reg_a = Register::from_val(348923042);
        ent(100, &Sign::Negative, &mut reg_a);
        assert_eq!(100, reg_a.val());
        let mut reg_a = Register::from_val(348923042);
        ent(0, &Sign::Negative, &mut reg_a);
        assert_eq!(
            Register::from_seq(Sign::Negative, &vec![0, 0, 0, 0, 0]),
            reg_a
        );
        let mut reg_a = Register::from_val(348923042);
        ent(0, &Sign::Positive, &mut reg_a);
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![0, 0, 0, 0, 0]),
            reg_a
        );

        let mut reg_a = Register::from_val(348923042);
        entn(100, &Sign::Negative, &mut reg_a);
        assert_eq!(-100, reg_a.val());
        let mut reg_a = Register::from_val(348923042);
        entn(0, &Sign::Negative, &mut reg_a);
        assert_eq!(
            Register::from_seq(Sign::Negative, &vec![0, 0, 0, 0, 0]),
            reg_a
        );
        let mut reg_a = Register::from_val(348923042);
        entn(0, &Sign::Positive, &mut reg_a);
        assert_eq!(
            Register::from_seq(Sign::Positive, &vec![0, 0, 0, 0, 0]),
            reg_a
        );

        let mut reg_a = Register::from_val(348923042);
        inc(10, &mut reg_a, &mut false);
        assert_eq!(348923042 + 10, reg_a.val());
        dec(10, &mut reg_a, &mut false);
        assert_eq!(348923042, reg_a.val());
    }
    #[test]
    fn test_cmp() {
        let reg_a = Register::from_val(100);
        let mem1000 = Register::from_val(200);
        assert_eq!(Ordering::Less, comp(&reg_a, &mem1000, Register::pair(0, 5)));
        assert_eq!(
            Ordering::Greater,
            comp(&reg_a, &mem1000, Register::pair(5, 5))
        );
        assert_eq!(
            Ordering::Equal,
            comp(&reg_a, &mem1000, Register::pair(1, 1))
        );
    }
    #[test]
    fn test_shift() {
        let mut reg_a = Register::from_seq(Sign::Positive, &vec![1, 2, 3, 4, 5]);
        let mut reg_x = Register::from_seq(Sign::Negative, &vec![6, 7, 8, 9, 10]);
        macro_rules! test {
            ($v1: expr, $v2: expr) => {
                assert_eq!(Register::from_seq(Sign::Positive, &$v1), reg_a);
                assert_eq!(Register::from_seq(Sign::Negative, &$v2), reg_x);
            };
        }
        shift_right_pair(1, &mut reg_a, &mut reg_x); // SRAX 1
        test!(vec![0, 1, 2, 3, 4], vec![5, 6, 7, 8, 9]);
        shift_left(2, &mut reg_a); // SLA 2
        test!(vec![2, 3, 4, 0, 0], vec![5, 6, 7, 8, 9]);
        rotate_right_pair(4, &mut reg_a, &mut reg_x); // SRC 4
        test!(vec![6, 7, 8, 9, 2], vec![3, 4, 0, 0, 5]);
        shift_right(2, &mut reg_a); // SRA 2
        test!(vec![0, 0, 6, 7, 8], vec![3, 4, 0, 0, 5]);
        rotate_left_pair(501, &mut reg_a, &mut reg_x); // SLC 501
        test!(vec![0, 6, 7, 8, 3], vec![4, 0, 0, 5, 0]);
    }
    #[test]
    fn test_conversion() {
        let mut reg_a = Register::from_seq(Sign::Negative, &vec![0, 0, 31, 32, 39]);
        let mut reg_x = Register::from_seq(Sign::Negative, &vec![37, 57, 47, 30, 30]);
        to_num(&mut reg_a, &reg_x, &mut false);
        assert_eq!(-12977700, reg_a.val());
        inc(1, &mut reg_a, &mut false);
        assert_eq!(-12977699, reg_a.val());
        to_char(&mut reg_a, &mut reg_x);
        assert_eq!(
            Register::from_seq(Sign::Negative, &vec![30, 30, 31, 32, 39]),
            reg_a
        );
        assert_eq!(
            Register::from_seq(Sign::Negative, &vec![37, 37, 36, 39, 39]),
            reg_x
        );
    }
    #[test]
    fn problem22() {
        macro_rules! solve1 {
            ($val: expr) => {
                let mem2000 = Register::from_val($val);
                let mut reg_a = Register::from_val(0);
                let mut reg_x = Register::from_val(0);
                ent(1, &Sign::Positive, &mut reg_a);
                for _ in 0..13 {
                    mul(&mem2000, &mut reg_a, &mut reg_x, Register::pair(0, 5));
                    rotate_right_pair(5, &mut reg_a, &mut reg_x);
                }
                assert_eq!($val.pow(13), reg_a.val());
            };
        }
        for i in (-4)..4 {
            solve1!(i);
        }

        macro_rules! solve2 {
            ($val: expr) => {
                let mut mem = vec![Register::from_val($val); 3];
                let mut reg_a = Register::from_val(0);
                let mut reg_x = Register::from_val(0);
                load(&mem[0], &mut reg_a, Register::pair(0, 5));
                mul(&mem[0], &mut reg_a, &mut reg_x, Register::pair(0, 5));
                store(&reg_x, &mut mem[1], Register::pair(0, 5));
                rotate_right_pair(5, &mut reg_a, &mut reg_x);
                mul(&mem[1], &mut reg_a, &mut reg_x, Register::pair(0, 5));
                store(&reg_x, &mut mem[2], Register::pair(0, 5));
                rotate_right_pair(5, &mut reg_a, &mut reg_x);
                mul(&mem[2], &mut reg_a, &mut reg_x, Register::pair(0, 5));
                rotate_right_pair(5, &mut reg_a, &mut reg_x);
                mul(&mem[2], &mut reg_a, &mut reg_x, Register::pair(0, 5));
                rotate_right_pair(5, &mut reg_a, &mut reg_x);
                mul(&mem[0], &mut reg_a, &mut reg_x, Register::pair(0, 5));
                rotate_right_pair(5, &mut reg_a, &mut reg_x);
                assert_eq!($val.pow(13), reg_a.val());
            };
        }
        for i in (-4)..4 {
            solve2!(i);
        }
    }
    #[test]
    fn problem23() {
        macro_rules! answer {
            ($v: expr) => {
                let mem = Register::from_seq(Sign::Positive, &$v);
                let mut reg_a = Register::from_val(0);
                let mut reg_x = Register::from_val(0);
                load(&mem, &mut reg_a, Register::pair(0, 5));
                shift_right(4, &mut reg_a);
                rotate_right_pair(1, &mut reg_a, &mut reg_x);
                load(&mem, &mut reg_a, Register::pair(0, 5));
                shift_right(3, &mut reg_a);
                rotate_right_pair(1, &mut reg_a, &mut reg_x);
                load(&mem, &mut reg_a, Register::pair(0, 5));
                shift_right(2, &mut reg_a);
                rotate_right_pair(1, &mut reg_a, &mut reg_x);
                load(&mem, &mut reg_a, Register::pair(0, 5));
                shift_right(1, &mut reg_a);
                rotate_right_pair(1, &mut reg_a, &mut reg_x);
                load(&mem, &mut reg_a, Register::pair(0, 5));
                shift_right(0, &mut reg_a);
                rotate_right_pair(1, &mut reg_a, &mut reg_x);
                shift_left_pair(5, &mut reg_a, &mut reg_x);

                assert_eq!(mem.byte(4), reg_a.byte(0));
                assert_eq!(mem.byte(3), reg_a.byte(1));
                assert_eq!(mem.byte(2), reg_a.byte(2));
                assert_eq!(mem.byte(1), reg_a.byte(3));
                assert_eq!(mem.byte(0), reg_a.byte(4));
            };
        }

        answer!(vec![1, 2, 3, 4, 5]);
    }
}
