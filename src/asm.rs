use crate::common::{instruction_data, Instruction, PseudoInstruction, CHAR_TABLE};
use crate::mix_word::{Byte, Sign, WordImpl};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
enum Attribute {
    Instruction(Instruction),
    PseudoInstruction(PseudoInstruction),
    Comment,
}

#[derive(Debug)]
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
fn tokenize(code: Vec<(usize, String)>) -> Vec<(usize, Content)> {
    let mut ret = vec![];

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
        let mut iter = code.split_whitespace().peekable();
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

    ret
}
fn resolve_symbol(mut code: Vec<(usize, Content)>) -> (usize, Vec<(usize, usize, Content)>) {
    use PseudoInstruction::*;

    // 1. resolve EQU
    // TODO: valueに式があれば解決する。
    let mut symbols = HashMap::new();
    for (
        _line,
        Content {
            attr,
            loc,
            addr,
            index,
            modification,
        },
    ) in &mut code
    {
        match attr {
            Attribute::PseudoInstruction(EQU) => {
                // EQUなのでsymbolsに突っ込む
                let variable = loc.as_ref().unwrap().clone().trim().to_string();
                let value = addr.clone().unwrap().trim().to_string();
                symbols.insert(variable, value);
            }
            Attribute::PseudoInstruction(ORIG)
            | Attribute::PseudoInstruction(CON)
            | Attribute::PseudoInstruction(END) => {
                if addr.is_some() {
                    let tmp_s = addr.clone().unwrap();
                    match symbols.get(&tmp_s) {
                        Some(value) => {
                            *addr = Some(value.to_string());
                        }
                        None => {}
                    }
                }
            }
            Attribute::Instruction(_) => {
                if addr.is_some() {
                    let tmp_s = addr.clone().unwrap();
                    match symbols.get(&tmp_s) {
                        Some(value) => {
                            *addr = Some(value.to_string());
                        }
                        None => {}
                    }
                }
                if index.is_some() {
                    let tmp_s = index.clone().unwrap();
                    match symbols.get(&tmp_s) {
                        Some(value) => {
                            *index = Some(value.to_string());
                        }
                        None => {}
                    }
                }
                if loc.is_some() {
                    let tmp_s = loc.clone().unwrap();
                    match symbols.get(&tmp_s) {
                        Some(value) => {
                            *loc = Some(value.to_string());
                        }
                        None => {}
                    }
                }
                if modification.is_some() {
                    let tmp_s = modification.clone().unwrap();
                    match symbols.get(&tmp_s) {
                        Some(value) => {
                            *modification = Some(value.to_string());
                        }
                        None => {}
                    }
                }
            }
            _ => {
                // comment or ALF
            }
        }
    }

    // 2. construct symbol table
    let mut symbols = SymbolTable {
        table: HashMap::new(),
    };
    let mut code_with_address = vec![];
    let mut address = 0usize;
    let mut entry_point = 0usize;
    for (line, content) in code {
        if content.loc.is_some() {
            symbols.push(content.loc.as_ref().unwrap().clone(), line, address);
        }
        match content.attr {
            Attribute::PseudoInstruction(ORIG) => {
                address = content.addr.unwrap().parse::<usize>().unwrap();
            }
            Attribute::PseudoInstruction(END) => {
                entry_point = symbols.table.get(&content.addr.unwrap()).unwrap()[0].1;
            }
            Attribute::PseudoInstruction(EQU) => {
                // do nothing
            }
            Attribute::Comment => {}
            _ => {
                code_with_address.push((line, address, content));
                address += 1;
            }
        }
    }

    // 3. resolve address
    for (line, address, content) in &mut code_with_address {
        match content.attr {
            Attribute::Instruction(_) => {
                if content.addr.is_some() && content.addr.as_ref().unwrap().parse::<i64>().is_err()
                {
                    let res = symbols.resolve(content.addr.as_ref().unwrap(), *line, *address);
                    content.addr = if res.is_some() {
                        Some(res.unwrap().to_string())
                    } else {
                        panic!()
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
    resolved_code: (usize, Vec<(usize, usize, Content)>),
) -> (usize, Vec<(usize, usize, WordImpl)>) {
    let (entry_point, code) = resolved_code;

    let mut ret = vec![];

    for (
        line,
        address,
        Content {
            attr,
            loc: _,
            addr,
            index,
            modification,
        },
    ) in code
    {
        match attr {
            Attribute::PseudoInstruction(pinst) => {
                if pinst == PseudoInstruction::CON {
                    let x = addr.unwrap().parse::<i64>().unwrap();
                    ret.push((line, address, WordImpl::from_val(x)));
                } else if pinst == PseudoInstruction::ALF {
                    let mut v = vec![];
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
                    a = addr.unwrap().parse::<i64>().unwrap();
                }
                if index.is_some() {
                    i = index.unwrap().parse::<i64>().unwrap();
                }
                if modification.is_some() {
                    let modification = modification.unwrap();
                    f = match modification.find(":") {
                        Some(_pos) => {
                            let mut tmp = modification.split(":");
                            let x = tmp.next().unwrap();
                            let y = tmp.next().unwrap();
                            x.trim().parse::<i64>().unwrap() * 8 + y.trim().parse::<i64>().unwrap()
                        }
                        None => modification.parse::<i64>().unwrap(),
                    }
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
    encode_to_binary(resolve_symbol(tokenize(split_by_line(code))))
}

#[derive(Debug)]
struct SymbolTable {
    table: HashMap<String, Vec<(usize, usize)>>,
}
fn eval(s: &str) -> i64 {
    match s.find("+") {
        Some(op_pos) => {
            let (lhs, rhs) = s.split_at(op_pos);
            lhs.parse::<i64>().unwrap() + rhs.parse::<i64>().unwrap()
        }
        None => match s.find("-") {
            Some(op_pos) => {
                let (lhs, rhs) = s.split_at(op_pos);
                lhs.parse::<i64>().unwrap() - rhs.parse::<i64>().unwrap()
            }
            None => s.parse().unwrap(),
        },
    }
}
impl SymbolTable {
    pub fn push(&mut self, variable: String, line: usize, address: usize) {
        match self.table.get_mut(&variable) {
            Some(v) => {
                (*v).push((line, address));
            }
            None => {
                self.table.insert(variable, vec![(line, address)]);
            }
        }
    }
    pub fn resolve(&self, variable: &str, line: usize, address: usize) -> Option<usize> {
        if variable.starts_with("*") {
            Some(eval(&variable.replace("*", &address.to_string())) as usize)
        } else if variable.starts_with(char::is_numeric) {
            // ローカルシンボルを解決
            let backward = variable.ends_with("B");
            let v = if backward {
                self.table.get(&variable.replace("B", "H")).unwrap()
            } else {
                self.table.get(&variable.replace("F", "H")).unwrap()
            };
            let mut ret = 9999;
            for (vline, vaddress) in v {
                if backward && vline < &line {
                    ret = *vaddress;
                } else if !backward && vline > &line && ret == 9999 {
                    ret = *vaddress;
                }
            }
            Some(ret)
        } else {
            match self.table.get(variable) {
                Some(v) => Some(v[0].1),
                None => None,
            }
        }
    }
}

fn format_finite_length(s: &str, len: usize) -> String {
    " ".repeat(len - s.len()).to_string() + s
}

pub fn format_code(code: String) -> String {
    use std::cmp::max;
    let tokenized_code = tokenize(split_by_line(code));
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
        let code = "* example program
                    * 
                    L EQU 500
                    PRINTER EQU 18
                    PRIME EQU -1
                    BUFO EQU 2000
                    BUF1 EQU 2025
                    ORIG 3000
                    START IOC 0(PRINTER)
                    ENT1 -499
                    ENT2 3
                    2H INC1 1
                    ST2 499,1
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
                    ENT4 2010
                    ENT5 -50
                    2H INC5 501
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
                    * initial contents of ...
                    ORIG 0
                    CON 2
                    ORIG 1995
                    TITLE ALF FISRT
                    ALF _FIVE
                    ALF _HUND
                    ALF RED_P
                    ALF RIMES
                    ORIG 2024
                    CON 2035
                    ORIG 2049
                    CON 2010
                    END START";
        let _tmp = encode_to_binary(resolve_symbol(tokenize(split_by_line(code.to_string()))));
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
        let _tmp = encode_to_binary(resolve_symbol(tokenize(split_by_line(code.to_string()))));
        let tmp = format_code(code.to_string());
        println!("\n{}", tmp);
    }
}
