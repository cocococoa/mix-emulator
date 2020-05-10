use crate::common::{instruction_data, Instruction, PseudoInstruction, CHAR_TABLE};
use crate::mix_word::{Byte, Sign, WordImpl};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Attribute {
    Instruction(Instruction),
    PseudoInstruction(PseudoInstruction),
    Comment,
}

#[derive(Debug, Clone)]
struct Content {
    attr: Attribute,
    loc: Option<String>,
    addr: Option<String>,
    index: Option<String>,
    modification: Option<String>,
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

fn split_by_line(code: String) -> Vec<(usize, String)> {
    let mut ret = vec![];

    for (l, content) in code.split_terminator('\n').enumerate() {
        ret.push((l, content.trim().to_string()));
    }

    ret
}
fn split_by_field(code: Vec<(usize, String)>) -> Vec<(usize, Content)> {
    use PseudoInstruction::*;

    let mut ret = vec![];
    let mut literal_constants: Vec<(usize, Content)> = vec![];
    for (line, code) in code {
        if code.starts_with("*") {
            ret.push((
                line,
                Content {
                    attr: Attribute::Comment,
                    loc: None,
                    addr: Some(code),
                    index: None,
                    modification: None,
                },
            ));
            continue;
        }
        let mut iter = code.split_whitespace();
        let first_term = iter.next().unwrap();
        let first_term_attribute = get_attribute(first_term);
        let (loc, attr) = if first_term_attribute.is_none() {
            // first term is loc
            let second_term = iter.next().unwrap();
            (
                Some(first_term.to_string()),
                get_attribute(second_term).unwrap(),
            )
        } else {
            (None, first_term_attribute.unwrap())
        };

        // rest == addr + index + modification
        let rest = iter.collect::<String>();
        if attr == Attribute::PseudoInstruction(END) {
            // move literal constants to ret
            for e in &literal_constants {
                ret.push(e.clone());
            }
        }
        if attr == Attribute::PseudoInstruction(ALF) {
            ret.push((
                line,
                Content {
                    attr: attr,
                    loc: loc,
                    addr: Some(rest),
                    index: None,
                    modification: None,
                },
            ))
        } else {
            if rest.starts_with("=") && rest.ends_with("=") {
                // "A" part is literal constant
                let unique_addr = "LC".to_string() + &literal_constants.len().to_string();
                ret.push((
                    line,
                    Content {
                        attr: attr,
                        loc: loc,
                        addr: Some(unique_addr.clone()),
                        index: None,
                        modification: None,
                    },
                ));
                let rest = rest.get(1..(rest.len() - 1)).unwrap().to_string();
                let (rest, modification) = match rest.find("(") {
                    Some(begin) => {
                        let end = rest.find(")").unwrap();
                        (
                            rest.get(..begin).unwrap().to_string(),
                            Some(rest.get((begin + 1)..end).unwrap().trim().to_string()),
                        )
                    }
                    None => (rest, None),
                };
                let addr = if rest.trim().len() != 0 {
                    Some(rest.trim().to_string())
                } else {
                    None
                };
                literal_constants.push((
                    9999,
                    Content {
                        attr: Attribute::PseudoInstruction(CON),
                        loc: Some(unique_addr),
                        addr: addr,
                        index: None,
                        modification: modification,
                    },
                ));
            } else {
                let (rest, modification) = match rest.find("(") {
                    Some(begin) => {
                        let end = rest.find(")").unwrap();
                        (
                            rest.get(..begin).unwrap().to_string(),
                            Some(rest.get((begin + 1)..end).unwrap().trim().to_string()),
                        )
                    }
                    None => (rest, None),
                };
                let (rest, index) = match rest.find(",") {
                    Some(begin) => (
                        rest.get(..begin).unwrap().to_string(),
                        Some(rest.get((begin + 1)..).unwrap().trim().to_string()),
                    ),
                    None => (rest, None),
                };
                let addr = if rest.trim().len() != 0 {
                    Some(rest.trim().to_string())
                } else {
                    None
                };
                ret.push((
                    line,
                    Content {
                        attr: attr,
                        loc: loc,
                        addr: addr,
                        index: index,
                        modification: modification,
                    },
                ))
            }
        }
    }

    ret
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

#[derive(Debug)]
struct ExpContent {
    attr: Attribute,
    loc: Option<String>,
    addr: Option<Exp>,
    index: Option<Exp>,
    modification: Option<Exp>,
}

fn evaluate_exp(exp: &Exp, symbols: &HashMap<String, i64>) -> Result<i64, ()> {
    use BinaryOp::*;

    let mut evaluated = match &exp.atom {
        AtomicExp::Asterisk => {
            return Err(());
        }
        AtomicExp::Symbol(s) => match symbols.get(s) {
            Some(v) => *v,
            None => return Err(()),
        },
        AtomicExp::Num(v) => *v,
    };
    evaluated = match exp.unary {
        UnaryOp::Plus => evaluated,
        UnaryOp::Minus => -evaluated,
    };

    for (binop, atom) in &exp.binop {
        let evaluated_atom = match atom {
            AtomicExp::Asterisk => {
                return Err(());
            }
            AtomicExp::Symbol(s) => match symbols.get(s) {
                Some(v) => *v,
                None => return Err(()),
            },
            AtomicExp::Num(v) => *v,
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
fn replace_asterisk(exp: &mut Exp, location_counter: usize) {
    match exp.atom {
        AtomicExp::Asterisk => {
            exp.atom = AtomicExp::Num(location_counter as i64);
        }
        _ => {}
    }
    for (_binop, atom) in &mut exp.binop {
        match atom {
            AtomicExp::Asterisk => {
                *atom = AtomicExp::Num(location_counter as i64);
            }
            _ => {}
        }
    }
}
fn replace_exp(exp: &mut Exp, symbols: &HashMap<String, i64>) {
    match &mut exp.atom {
        AtomicExp::Symbol(s) => {
            let v = symbols.get(s);
            if v.is_some() {
                exp.atom = AtomicExp::Num(*v.unwrap());
            }
        }
        _ => {}
    }
    for (_binop, atom) in &mut exp.binop {
        match atom {
            AtomicExp::Symbol(s) => {
                let v = symbols.get(s);
                if v.is_some() {
                    *atom = AtomicExp::Num(*v.unwrap());
                }
            }
            _ => {}
        }
    }
}

fn is_local_symbol(s: &str) -> Option<usize> {
    if s.ends_with("H") || s.ends_with("F") || s.ends_with("B") {
        let num_end = s.len() - 1;
        match s.get(..num_end).unwrap().parse::<usize>() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    } else {
        None
    }
}

fn get_symbol(e: &Exp) -> Option<String> {
    match &e.atom {
        AtomicExp::Symbol(s) => Some(s.clone()),
        _ => None,
    }
}

fn make_num_exp(n: i64) -> Exp {
    Exp {
        unary: if n >= 0 {
            UnaryOp::Plus
        } else {
            UnaryOp::Minus
        },
        atom: AtomicExp::Num(n.abs()),
        binop: vec![],
    }
}

// TODO: implement W-value
fn resolve_symbol(code: Vec<(usize, Content)>) -> (usize, Vec<(usize, usize, ExpContent)>) {
    use PseudoInstruction::*;

    // 0. construct expression
    let mut expressed_code = vec![];
    for (
        line,
        Content {
            attr,
            loc,
            addr,
            index,
            modification,
        },
    ) in code
    {
        match attr {
            Attribute::Comment => {}
            Attribute::PseudoInstruction(ALF) => expressed_code.push((
                line,
                ExpContent {
                    attr: attr,
                    loc: loc,
                    addr: Some(Exp {
                        unary: UnaryOp::Plus,
                        atom: AtomicExp::Symbol(addr.unwrap()),
                        binop: vec![],
                    }),
                    index: None,
                    modification: None,
                },
            )),
            _ => expressed_code.push((
                line,
                ExpContent {
                    attr: attr,
                    loc: loc,
                    addr: if addr.is_some() {
                        Some(construct_exp(&addr.unwrap()))
                    } else {
                        None
                    },
                    index: if index.is_some() {
                        Some(construct_exp(&index.unwrap()))
                    } else {
                        None
                    },
                    modification: if modification.is_some() {
                        Some(construct_exp(&modification.unwrap()))
                    } else {
                        None
                    },
                },
            )),
        }
    }

    // 1. resolve EQU
    let mut symbols = HashMap::new();
    for (
        _line,
        ExpContent {
            attr,
            loc,
            addr,
            index,
            modification,
        },
    ) in &mut expressed_code
    {
        // println!(
        //     "[stage1] line: {:2}, attr: {:?}, loc: {:?}, addr: {:?}, index: {:?}, modi: {:?}",
        //     _line, attr, loc, addr, index, modification
        // );
        match attr {
            Attribute::PseudoInstruction(EQU) => {
                let variable = loc.as_ref().unwrap().clone().to_string();
                let addr_value = evaluate_exp(addr.as_ref().unwrap(), &symbols).unwrap();
                let value = if modification.is_some() {
                    let modification_value =
                        evaluate_exp(modification.as_ref().unwrap(), &symbols).unwrap();
                    WordImpl::from_val(addr_value)
                        .subword(modification_value as usize)
                        .unwrap()
                        .val()
                } else {
                    addr_value
                };
                symbols.insert(variable, value);
            }
            Attribute::PseudoInstruction(ORIG)
            | Attribute::PseudoInstruction(CON)
            | Attribute::PseudoInstruction(END) => {
                if addr.is_some() {
                    replace_exp(&mut addr.as_mut().unwrap(), &symbols);
                }
                if modification.is_some() {
                    replace_exp(&mut modification.as_mut().unwrap(), &symbols);
                }
            }
            Attribute::Instruction(_) => {
                if addr.is_some() {
                    replace_exp(&mut addr.as_mut().unwrap(), &symbols);
                }
                if index.is_some() {
                    replace_exp(&mut index.as_mut().unwrap(), &symbols);
                }
                if modification.is_some() {
                    replace_exp(&mut modification.as_mut().unwrap(), &symbols);
                }
            }
            _ => {
                // comment or ALF
            }
        }
    }

    // 2. construct symbol table
    let mut local_symbols = SymbolTable {
        table: HashMap::new(),
    };
    let mut code_with_address = vec![];
    let mut location_counter = 0usize;
    let mut entry_point = 0usize;
    for (line, mut content) in expressed_code {
        // println!(
        //     "[stage2] line: {:2}, attr: {:?}, loc: {:?}, addr: {:?}, index: {:?}, modi: {:?}",
        //     line, content.attr, content.loc, content.addr, content.index, content.modification
        // );
        if content.loc.is_some() {
            match is_local_symbol(&content.loc.as_ref().unwrap()) {
                Some(value) => local_symbols.push(value, line, location_counter),
                None => {
                    let _ = symbols.insert(
                        content.loc.as_ref().unwrap().clone(),
                        location_counter as i64,
                    );
                }
            }
        }
        match content.attr {
            Attribute::PseudoInstruction(ORIG) => {
                replace_asterisk(&mut content.addr.as_mut().unwrap(), location_counter);
                location_counter = evaluate_exp(&content.addr.unwrap(), &symbols).unwrap() as usize;
            }
            Attribute::PseudoInstruction(END) => {
                replace_asterisk(&mut content.addr.as_mut().unwrap(), location_counter);
                entry_point = evaluate_exp(&content.addr.unwrap(), &symbols).unwrap() as usize;
                break;
            }
            Attribute::PseudoInstruction(EQU) => {}
            Attribute::Comment => {}
            _ => {
                if content.addr.is_some() {
                    replace_asterisk(&mut content.addr.as_mut().unwrap(), location_counter);
                }
                code_with_address.push((line, location_counter, content));
                location_counter += 1;
            }
        }
    }
    // println!("{:?}", symbols);
    // 3. resolve address
    for (line, _address, content) in &mut code_with_address {
        // println!(
        //     "[stage3] line: {:2}, attr: {:?}, loc: {:?}, addr: {:?}, index: {:?}, modi: {:?}",
        //     line, content.attr, content.loc, content.addr, content.index, content.modification
        // );
        match content.attr {
            Attribute::Instruction(_) | Attribute::PseudoInstruction(CON) => {
                // TODO: implement more
                if content.addr.is_some() {
                    replace_exp(&mut content.addr.as_mut().unwrap(), &symbols);
                    if evaluate_exp(content.addr.as_ref().unwrap(), &symbols).is_err() {
                        let addr = get_symbol(content.addr.as_ref().unwrap()).unwrap();
                        match is_local_symbol(&addr) {
                            Some(v) => {
                                // resolve local symbol
                                *content.addr.as_mut().unwrap() = if addr.ends_with("F") {
                                    make_num_exp(local_symbols.resolve_forward(v, *line) as i64)
                                } else {
                                    make_num_exp(local_symbols.resolve_backward(v, *line) as i64)
                                }
                            }
                            None => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    (entry_point, code_with_address)
}

fn char_to_num(c: char) -> Option<usize> {
    for (i, mix_c) in CHAR_TABLE.iter().enumerate() {
        if c == *mix_c {
            return Some(i);
        }
    }
    None
}

fn encode_to_binary(
    resolved_code: (usize, Vec<(usize, usize, ExpContent)>),
) -> (usize, Vec<(usize, usize, WordImpl)>) {
    use PseudoInstruction::*;

    let (entry_point, code) = resolved_code;

    let mut ret = vec![];
    let symbols = HashMap::new();

    for (
        line,
        address,
        ExpContent {
            attr,
            loc: _loc,
            addr,
            index,
            modification,
        },
    ) in code
    {
        // println!(
        //     "[binary] line: {:2}, attr: {:?}, loc: {:?}, addr: {:?}, index: {:?}, modi: {:?}",
        //     line, attr, _loc, addr, index, modification
        // );
        match attr {
            Attribute::PseudoInstruction(pinst) => {
                if pinst == CON {
                    let x = evaluate_exp(&addr.unwrap(), &symbols);
                    ret.push((line, address, WordImpl::from_val(x.unwrap())));
                } else if pinst == ALF {
                    let mut v = vec![];
                    let addr = get_symbol(&addr.unwrap());
                    for c in addr.unwrap().replace("_", " ").chars() {
                        v.push(char_to_num(c).unwrap() as u32);
                    }
                    ret.push((line, address, WordImpl::from_seq(Sign::Positive, &v)));
                } else {
                    unreachable!();
                }
            }
            Attribute::Instruction(inst) => {
                let (mut a, mut i, mut f, c) = instruction_data(&inst);
                if addr.is_some() {
                    a = evaluate_exp(&addr.unwrap(), &symbols).unwrap();
                }
                if index.is_some() {
                    i = evaluate_exp(&index.unwrap(), &symbols).unwrap();
                }
                if modification.is_some() {
                    f = evaluate_exp(&modification.unwrap(), &symbols).unwrap();
                }
                let sign = if a < 0 {
                    a = a.abs();
                    Sign::Negative
                } else {
                    Sign::Positive
                };

                ret.push((
                    line,
                    address,
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
                ))
            }
            _ => {
                unreachable!();
            }
        }
    }

    (entry_point, ret)
}

pub fn assemble(code: String) -> (usize, Vec<(usize, usize, WordImpl)>) {
    encode_to_binary(resolve_symbol(split_by_field(split_by_line(code))))
}

#[derive(Debug)]
struct SymbolTable {
    table: HashMap<usize, Vec<(usize, usize)>>,
}
impl SymbolTable {
    pub fn push(&mut self, variable: usize, line: usize, address: usize) {
        match self.table.get_mut(&variable) {
            Some(v) => {
                (*v).push((line, address));
            }
            None => {
                self.table.insert(variable, vec![(line, address)]);
            }
        }
    }
    pub fn resolve_backward(&self, variable: usize, line: usize) -> usize {
        let v = self.table.get(&variable).unwrap();
        v.iter()
            .filter_map(|e| if e.0 < line { Some(e.1) } else { None })
            .collect::<Vec<_>>()
            .pop()
            .unwrap()
    }
    pub fn resolve_forward(&self, variable: usize, line: usize) -> usize {
        let v = self.table.get(&variable).unwrap();
        v.iter()
            .filter_map(|e| if e.0 > line { Some(e.1) } else { None })
            .collect::<Vec<_>>()[0]
    }
}

fn format_finite_length(s: &str, len: usize) -> String {
    " ".repeat(len - s.len()).to_string() + s
}

pub fn format_code(code: String) -> String {
    use std::cmp::max;
    let tokenized_code = split_by_field(split_by_line(code));
    let mut loc_len = 0usize;
    let mut a_len = 0usize;
    let mut i_len = 0usize;
    let mut f_len = 0usize;
    for (
        _line,
        Content {
            attr,
            loc,
            addr,
            index,
            modification,
        },
    ) in &tokenized_code
    {
        if attr != &Attribute::Comment {
            if loc.is_some() {
                loc_len = max(loc_len, loc.as_ref().unwrap().len());
            }
            if addr.is_some() {
                a_len = max(a_len, addr.as_ref().unwrap().len())
            }
            if index.is_some() {
                i_len = max(i_len, index.as_ref().unwrap().len())
            }
            if modification.is_some() {
                f_len = max(f_len, modification.as_ref().unwrap().len())
            }
        }
    }

    // 遊びを持たせる
    loc_len += 3;
    a_len += 3;
    i_len += 3;
    f_len += 3;

    let mut ret = "".to_string();

    ret.push_str(" | ");
    ret.push_str("LINE");
    ret.push_str(" | ");
    ret.push_str(&format_finite_length("LOC", loc_len));
    ret.push_str(" | ");
    ret.push_str("   OPE");
    ret.push_str(" | ");
    ret.push_str(&format_finite_length("A", a_len));
    ret.push_str(" | ");
    ret.push_str(&format_finite_length("I", i_len));
    ret.push_str(" | ");
    ret.push_str(&format_finite_length("F", f_len));
    ret.push_str(" | ");
    ret.push('\n');
    for (
        line,
        Content {
            attr,
            loc,
            addr,
            index,
            modification,
        },
    ) in tokenized_code
    {
        let li = format!("{:4}", line);
        ret.push_str(" | ");
        ret.push_str(&li);
        ret.push_str(" | ");
        if attr == Attribute::Comment {
            ret.push_str(&addr.unwrap());
            ret.push('\n');
        } else {
            let lo = if loc.is_some() {
                format_finite_length(&loc.unwrap(), loc_len)
            } else {
                " ".repeat(loc_len).to_string()
            };
            let op = match attr {
                Attribute::Instruction(inst) => format_finite_length(&inst.to_string(), 6),
                Attribute::PseudoInstruction(pinst) => format_finite_length(&pinst.to_string(), 6),
                _ => "".to_string(),
            };
            let a = if addr.is_some() {
                format_finite_length(addr.as_ref().unwrap(), a_len)
            } else {
                " ".repeat(a_len).to_string()
            };
            let i = if index.is_some() {
                format_finite_length(index.as_ref().unwrap(), i_len)
            } else {
                " ".repeat(i_len).to_string()
            };
            let f = if modification.is_some() {
                format_finite_length(modification.as_ref().unwrap(), f_len)
            } else {
                " ".repeat(f_len).to_string()
            };
            ret.push_str(&lo);
            ret.push_str(" | ");
            ret.push_str(&op);
            ret.push_str(" | ");
            ret.push_str(&a);
            ret.push_str(" | ");
            ret.push_str(&i);
            ret.push_str(" | ");
            ret.push_str(&f);
            ret.push_str(" | ");
            ret.push('\n');
        }
    }

    ret
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
        let _tmp = encode_to_binary(resolve_symbol(split_by_field(split_by_line(
            code.to_string(),
        ))));
        let tmp = format_code(code.to_string());
        println!("\n{}", tmp);
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
        let _tmp = encode_to_binary(resolve_symbol(split_by_field(split_by_line(
            code.to_string(),
        ))));
        let tmp = format_code(code.to_string());
        println!("\n{}", tmp);
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
