use crate::common::{instruction_data, Instruction, PseudoInstruction, CHAR_TABLE};
use crate::mix_word::{Byte, Sign, WordImpl};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
enum Attribute {
    Instruction(Instruction),
    PseudoInstruction(PseudoInstruction),
}

fn to_instruction(s: &str) -> Option<Instruction> {
    match Instruction::from_str(s) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}
fn to_pseudo_instruction(s: &str) -> Option<PseudoInstruction> {
    match PseudoInstruction::from_str(s) {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}
fn get_attribute(s: &str) -> Option<Attribute> {
    let a = to_instruction(s);
    let b = to_pseudo_instruction(s);

    if a.is_some() {
        Some(Attribute::Instruction(a.unwrap()))
    } else if b.is_some() {
        Some(Attribute::PseudoInstruction(b.unwrap()))
    } else {
        None
    }
}
fn char_to_num(c: char) -> Option<usize> {
    for (i, mix_c) in CHAR_TABLE.iter().enumerate() {
        if c == *mix_c {
            return Some(i);
        }
    }
    None
}
fn split_into_loc_ope_addr(code: &str) -> (Option<&str>, Attribute, &str) {
    // TODO: temporarily, assumes no whitespaces in code
    let mut iter = code.split_whitespace();
    let first_term = iter.next().unwrap();
    let first_term_attribute = get_attribute(first_term);
    let (loc, attr) = if first_term_attribute.is_none() {
        // first term is loc
        let second_term = iter.next().unwrap();
        (Some(first_term), get_attribute(second_term).unwrap())
    } else {
        // first term is ope
        // ERROR: if no operation in code
        (None, first_term_attribute.unwrap())
    };
    let addr = iter.next().unwrap_or("0");

    (loc, attr, addr)
}
fn split_into_addr_index_modi(code: &str) -> (&str, Option<&str>, Option<&str>) {
    let (code, modi) = match code.find('(') {
        Some(mid) => {
            if !code.ends_with(")") {
                // ERROR: should end with ")"
                panic!();
            }
            (
                code.get(..mid).unwrap(),
                code.get((mid + 1)..(code.len() - 1)),
            )
        }
        None => (code, None),
    };
    let (addr, index) = match code.find(",") {
        Some(mid) => (code.get(..mid).unwrap(), code.get((mid + 1)..)),
        None => (code, None),
    };
    (addr, index, modi)
}
// here
fn is_local_symbol_h(s: &str) -> bool {
    if s.ends_with("H") {
        let digit_end = s.len() - 1;
        s.get(..digit_end).unwrap().parse::<usize>().is_ok()
    } else {
        false
    }
}
// forward
fn is_local_symbol_f(s: &str) -> bool {
    if s.ends_with("F") {
        let digit_end = s.len() - 1;
        s.get(..digit_end).unwrap().parse::<usize>().is_ok()
    } else {
        false
    }
}
// backward
fn is_local_symbol_b(s: &str) -> bool {
    if s.ends_with("B") {
        let digit_end = s.len() - 1;
        s.get(..digit_end).unwrap().parse::<usize>().is_ok()
    } else {
        false
    }
}

pub fn debug_assemble(code: &str) -> (usize, Vec<(usize, usize, WordImpl)>) {
    use PseudoInstruction::*;

    // return value
    let mut binary: Vec<(usize, usize, WordImpl)> = vec![];
    let mut entry_point = 0usize;

    // tables
    let mut symbol_table: HashMap<String, i64> = HashMap::new();
    let mut unresolved_symbol: HashMap<String, Vec<usize>> = HashMap::new();
    let mut constant_literal: Vec<(String, String)> = vec![];

    // generated codes
    let mut generated_code: Vec<String> = vec![];

    // loop indices
    let mut line_iterator = code.lines().enumerate().peekable();
    let mut location_counter = 0usize;

    loop {
        if line_iterator.peek().is_none() {
            break;
        }
        // check whether this line is END or not
        let (_line, content) = line_iterator.peek().unwrap();
        let content = content.trim();
        if content.starts_with("*") {
            // comment
            let _ = line_iterator.next();
            continue;
        }
        let (_loc, attr, _addr) = split_into_loc_ope_addr(content);

        let (line, content) = if attr == Attribute::PseudoInstruction(END) {
            // if END, generate codes
            if constant_literal.len() != 0 {
                let (unique_symbol, addr) = constant_literal.pop().unwrap();
                generated_code.push(unique_symbol + " CON " + &addr);
                // TODO: remove magic number 7777
                (7777, generated_code.last().unwrap().as_str())
            } else if unresolved_symbol.len() != 0 {
                let loc = unresolved_symbol.keys().next().unwrap();
                generated_code.push(loc.clone() + " CON 0");
                (7777, generated_code.last().unwrap().as_str())
            } else {
                // all tables are clear
                line_iterator.next().unwrap()
            }
        } else {
            // if not END, step iterator
            line_iterator.next().unwrap()
        };

        // 1. split content into LOC, OPE, ADDR
        let (loc, attr, addr) = split_into_loc_ope_addr(content);

        // 2. push LOC into HashMap
        if attr != Attribute::PseudoInstruction(EQU) && loc.is_some() {
            let loc = loc.unwrap();
            let loc = if is_local_symbol_h(loc) {
                loc.get(..(loc.len() - 1)).unwrap()
            } else {
                loc
            };
            // search unresolved symbols and resolve them
            let unresolved = unresolved_symbol.remove(loc);
            if unresolved.is_some() {
                let unresolved = unresolved.unwrap();
                for pos in unresolved.iter() {
                    let word = &mut binary.get_mut(*pos).unwrap().2;
                    *word.byte_mut(0).unwrap() =
                        Byte::new((location_counter / Byte::max() as usize) as u32);
                    *word.byte_mut(1).unwrap() =
                        Byte::new((location_counter % Byte::max() as usize) as u32);
                }
            }
            // if loc is local symbol, remove from symbol_table
            if loc.parse::<usize>().is_ok() {
                let _ = symbol_table.remove(loc);
            }
            if symbol_table.get(loc).is_some() {
                // ERROR: cannot define twice
                panic!();
            }
            symbol_table.insert(loc.to_string(), location_counter as i64);
        }

        // 3. deal with ALF
        if attr == Attribute::PseudoInstruction(ALF) {
            if addr.len() != 5 {
                // ERROR: addr length should be equal to 5
                panic!();
            }
            let mut v = vec![];
            for c in addr.replace("_", " ").chars() {
                match char_to_num(c) {
                    Some(num) => v.push(num as u32),
                    // ERROR: illegal char in addr
                    None => panic!(),
                }
            }
            binary.push((
                line,
                location_counter,
                WordImpl::from_seq(Sign::Positive, &v),
            ));
            location_counter += 1;
            continue;
        }

        // 4. deal with literal constants
        let addr = if addr.starts_with("=") {
            if !addr.ends_with("=") {
                // ERROR: should end with "="
                panic!();
            }
            // TODO: generate real unique symbol
            let unique_symbol = "UNQSYM".to_string() + &constant_literal.len().to_string();
            constant_literal.push((
                unique_symbol,
                addr.get(1..(addr.len() - 1)).unwrap().to_string(),
            ));

            &constant_literal.last().as_ref().unwrap().0
        } else {
            addr
        };

        // 5. split addr into ADDR, INDEX, MODIFICATION and construct expression
        // and resolve symbols
        // TODO: temporarily, ignore W-value
        let (addr, index, modi) = split_into_addr_index_modi(addr);
        let index = index
            .map(|s| construct_exp(s))
            .map(|exp| replace_symbol(exp, &symbol_table))
            .map(|exp| replace_asterisk(exp, location_counter))
            .map(|exp| eval(exp).unwrap());
        let modi = modi
            .map(|s| construct_exp(s))
            .map(|exp| replace_symbol(exp, &symbol_table))
            .map(|exp| replace_asterisk(exp, location_counter))
            .map(|exp| eval(exp).unwrap());
        // TODO: remove Some(..).map()....unwrap() pattern
        let mut addr = Some(addr)
            .map(|s| construct_exp(s))
            .map(|exp| replace_symbol(exp, &symbol_table))
            .map(|exp| replace_asterisk(exp, location_counter))
            .unwrap();

        // 6. if addr contains unresolved symbol, insert to unresolve_symbol HashMap
        if get_unresolved_symbol(&addr).is_some() {
            // ERROR: addr with unresolved symbol has empty binop
            assert_eq!(addr.binop, vec![]);

            let mut s = get_unresolved_symbol(&addr).unwrap();
            if is_local_symbol_f(s.as_ref()) {
                s.pop();
            }
            match unresolved_symbol.get_mut(&s) {
                Some(v) => {
                    v.push(binary.len());
                }
                None => {
                    unresolved_symbol.insert(s, vec![binary.len()]);
                }
            }

            // assign zero for temp.
            addr = Exp {
                unary: UnaryOp::Plus,
                atom: AtomicExp::Num(0),
                binop: vec![],
            };
        };

        // 7. addr is evaluatable
        let addr = eval(addr).unwrap();

        // 8. finalize (encode to binary)
        match attr {
            Attribute::Instruction(inst) => {
                let (_a, i, f, c) = instruction_data(&inst);
                let mut a = addr;
                let i = index.unwrap_or(i as i64);
                let f = modi.unwrap_or(f as i64);
                let sign = if a < 0 {
                    a = a.abs();
                    Sign::Negative
                } else {
                    Sign::Positive
                };

                binary.push((
                    line,
                    location_counter,
                    WordImpl::from_seq(
                        sign,
                        &vec![
                            a as u32 / Byte::max(),
                            a as u32 % Byte::max(),
                            i as u32,
                            f as u32,
                            c as u32,
                        ],
                    ),
                ));
                location_counter += 1;
            }
            Attribute::PseudoInstruction(EQU) => {
                assert!(index.is_none());
                let var = loc.unwrap();
                let val = weval(addr, modi.unwrap_or(5));
                symbol_table.insert(var.to_string(), val);
            }
            Attribute::PseudoInstruction(ORIG) => {
                assert!(index.is_none());
                location_counter = weval(addr, modi.unwrap_or(5)) as usize;
            }
            Attribute::PseudoInstruction(CON) => {
                assert!(index.is_none());
                binary.push((
                    line,
                    location_counter,
                    WordImpl::from_val(weval(addr, modi.unwrap_or(5))),
                ));
                location_counter += 1;
            }
            Attribute::PseudoInstruction(END) => {
                assert!(index.is_none());
                let val = weval(addr, modi.unwrap_or(5));
                entry_point = val as usize;
            }
            Attribute::PseudoInstruction(ALF) => {
                unreachable!();
            }
        }
    } // main loop

    (entry_point, binary)
}

#[derive(Debug, PartialEq, Eq)]
enum AtomicExp {
    Num(i64),
    Symbol(String),
    Asterisk,
}
#[derive(Debug, PartialEq, Eq)]
enum UnaryOp {
    Plus,
    Minus,
}
#[derive(Debug, PartialEq, Eq)]
enum BinaryOp {
    ADD,
    SUB,
    MUL,
    DIV,
    FRAC,
    COLON,
}
#[derive(Debug, PartialEq, Eq)]
struct Exp {
    unary: UnaryOp,
    atom: AtomicExp,
    binop: Vec<(BinaryOp, AtomicExp)>,
}
const SYMBOL: [char; 5] = ['+', '-', '*', '/', ':'];
fn consume_atomic(code: &str) -> (AtomicExp, usize) {
    if code.chars().nth(0).unwrap() == '*' {
        (AtomicExp::Asterisk, 1)
    } else {
        let mut end = 0;
        for c in code.chars() {
            if SYMBOL.contains(&c) {
                break;
            }
            end += 1;
        }
        let atom_string = code.get(..end).unwrap();
        let atom = match atom_string.parse::<i64>() {
            Ok(v) => AtomicExp::Num(v),
            Err(_) => AtomicExp::Symbol(atom_string.to_string()),
        };

        (atom, end)
    }
}
fn consume_binop(code: &str) -> (BinaryOp, usize) {
    match code.chars().nth(0).unwrap() {
        '+' => (BinaryOp::ADD, 1),
        '-' => (BinaryOp::SUB, 1),
        '*' => (BinaryOp::MUL, 1),
        ':' => (BinaryOp::COLON, 1),
        '/' => {
            if code.starts_with("//") {
                (BinaryOp::FRAC, 2)
            } else {
                (BinaryOp::DIV, 1)
            }
        }
        _ => unreachable!(),
    }
}
fn construct_exp(code: &str) -> Exp {
    // TODO: temporarily, assumes no whitespaces in code
    let (unary, mut look_at) = if code.chars().nth(0).unwrap() == '+' {
        (UnaryOp::Plus, 1)
    } else if code.chars().nth(0).unwrap() == '-' {
        (UnaryOp::Minus, 1)
    } else {
        (UnaryOp::Plus, 0)
    };
    let (atom, diff) = consume_atomic(code.get(look_at..).unwrap());
    look_at += diff;

    let mut binop = vec![];
    loop {
        // binopが続くか判断
        if look_at == code.len() {
            break;
        }
        let (bin, diff) = consume_binop(code.get(look_at..).unwrap());
        look_at += diff;
        let (atom2, diff) = consume_atomic(code.get(look_at..).unwrap());
        look_at += diff;
        binop.push((bin, atom2));
    }

    Exp {
        unary: unary,
        atom: atom,
        binop: binop,
    }
}
fn replace_asterisk_impl(atom: AtomicExp, location_counter: usize) -> AtomicExp {
    match atom {
        AtomicExp::Asterisk => AtomicExp::Num(location_counter as i64),
        _ => atom,
    }
}
fn replace_asterisk(exp: Exp, location_counter: usize) -> Exp {
    let unary = exp.unary;
    let atom = replace_asterisk_impl(exp.atom, location_counter);
    let mut binop = vec![];
    for (op, atom) in exp.binop {
        binop.push((op, replace_asterisk_impl(atom, location_counter)));
    }

    Exp {
        unary: unary,
        atom: atom,
        binop: binop,
    }
}
fn replace_symbol_impl(atom: AtomicExp, symbols: &HashMap<String, i64>) -> AtomicExp {
    match &atom {
        AtomicExp::Symbol(s) => {
            // TODO: move is_local_symbol_b to outside of this func.
            let s = if is_local_symbol_b(s.as_str()) {
                s.get(..(s.len() - 1)).unwrap()
            } else {
                s
            };
            symbols.get(s).map(|v| AtomicExp::Num(*v)).unwrap_or(atom)
        }
        _ => atom,
    }
}
fn replace_symbol(exp: Exp, symbols: &HashMap<String, i64>) -> Exp {
    let unary = exp.unary;
    let atom = replace_symbol_impl(exp.atom, &symbols);
    let mut binop = vec![];
    for (op, atom) in exp.binop {
        binop.push((op, replace_symbol_impl(atom, &symbols)));
    }

    Exp {
        unary: unary,
        atom: atom,
        binop: binop,
    }
}
fn get_unresolved_symbol(e: &Exp) -> Option<String> {
    match &e.atom {
        AtomicExp::Symbol(s) => Some(s.clone()),
        _ => None,
    }
}
fn eval(exp: Exp) -> Result<i64, ()> {
    use BinaryOp::*;

    let mut evaluated = match exp.atom {
        AtomicExp::Num(v) => v,
        _ => return Err(()),
    };
    evaluated = match exp.unary {
        UnaryOp::Plus => evaluated,
        UnaryOp::Minus => -evaluated,
    };

    for (binop, atom) in exp.binop {
        let evaluated_atom = match atom {
            AtomicExp::Num(v) => v,
            _ => return Err(()),
        };

        match binop {
            ADD => evaluated += evaluated_atom,
            SUB => evaluated -= evaluated_atom,
            MUL => evaluated *= evaluated_atom,
            DIV => evaluated /= evaluated_atom,
            FRAC => evaluated = evaluated * Byte::word_max() / evaluated_atom,
            COLON => evaluated = 8 * evaluated + evaluated_atom,
        }
    }

    Ok(evaluated)
}
fn weval(a: i64, f: i64) -> i64 {
    // TODO: evaluate real W-value in the future
    WordImpl::from_val(a).subword(f as usize).unwrap().val()
}

pub fn release_assemble(code: &str) -> (usize, Vec<(usize, WordImpl)>) {
    let (entry_point, dbinary) = debug_assemble(code);

    (
        entry_point,
        dbinary.into_iter().map(|l| (l.1, l.2)).collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_prime() {
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
                    2H OUT TITLE(PRINTER)
                    ENT4 BUF1+10
                    ENT5 -50
                    2H INC5 L+1
                    4H LDA PRIME,5
                    CHAR
                    STX 0,4(1:4)
                    DEC4 1
                    DEC5 50
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
                    CON BUF1+10
                    ORIG BUF1+24
                    CON BUF0+10
                    END START";
        debug_assemble(code);
    }
    #[test]
    fn test_max() {
        let code = "X EQU 1000
                    ORIG 3000
                    START STJ EXIT
                    INIT ENT3 0,1
                    JMP CHANGEM
                    LOOP CMPA X,3
                    JGE *+3
                    CHANGEM ENT2 0,3
                    LDA X,3
                    DEC3 1
                    J3P LOOP
                    EXIT JMP *
                    END START";
        debug_assemble(code);
    }
    #[test]
    fn test_expression() {
        let s = "20YF20+*-100";
        assert_eq!(
            Exp {
                unary: UnaryOp::Plus,
                atom: AtomicExp::Symbol("20YF20".to_string()),
                binop: vec![
                    (BinaryOp::ADD, AtomicExp::Asterisk),
                    (BinaryOp::SUB, AtomicExp::Num(100))
                ]
            },
            construct_exp(s)
        );
        let s = "-1+5";
        assert_eq!(
            Exp {
                unary: UnaryOp::Minus,
                atom: AtomicExp::Num(1),
                binop: vec![(BinaryOp::ADD, AtomicExp::Num(5)),]
            },
            construct_exp(s)
        );
        let s = "-1+5*20/6";
        assert_eq!(
            Exp {
                unary: UnaryOp::Minus,
                atom: AtomicExp::Num(1),
                binop: vec![
                    (BinaryOp::ADD, AtomicExp::Num(5)),
                    (BinaryOp::MUL, AtomicExp::Num(20)),
                    (BinaryOp::DIV, AtomicExp::Num(6)),
                ]
            },
            construct_exp(s)
        );
        let s = "1//3";
        assert_eq!(
            Exp {
                unary: UnaryOp::Plus,
                atom: AtomicExp::Num(1),
                binop: vec![(BinaryOp::FRAC, AtomicExp::Num(3)),]
            },
            construct_exp(s)
        );
        let s = "1:3";
        assert_eq!(
            Exp {
                unary: UnaryOp::Plus,
                atom: AtomicExp::Num(1),
                binop: vec![(BinaryOp::COLON, AtomicExp::Num(3)),]
            },
            construct_exp(s)
        );
        let s = "*-3";
        assert_eq!(
            Exp {
                unary: UnaryOp::Plus,
                atom: AtomicExp::Asterisk,
                binop: vec![(BinaryOp::SUB, AtomicExp::Num(3)),]
            },
            construct_exp(s)
        );
        let s = "***";
        assert_eq!(
            Exp {
                unary: UnaryOp::Plus,
                atom: AtomicExp::Asterisk,
                binop: vec![(BinaryOp::MUL, AtomicExp::Asterisk),]
            },
            construct_exp(s)
        );
    }
}
