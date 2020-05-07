use crate::common::{instruction_data, Instruction, PseudoInstruction, CHAR_TABLE};
use crate::mix_word::{Byte, Sign, WordImpl};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
enum Attribute {
    Instruction(Instruction),
    PseudoInstruction(PseudoInstruction),
}
#[derive(Debug)]
enum Content {
    Instruction(Instruction, Option<String>, Option<String>, Option<String>),
    PseudoInstruction(PseudoInstruction, Option<String>, Option<String>),
}

fn to_instruction(s: &str) -> Option<Instruction> {
    for slice in s.split_whitespace() {
        match Instruction::from_str(slice) {
            Ok(v) => {
                return Some(v);
            }
            Err(_) => {}
        }
    }
    None
}
fn to_pseudo_instruction(s: &str) -> Option<PseudoInstruction> {
    for slice in s.split_whitespace() {
        match PseudoInstruction::from_str(slice) {
            Ok(v) => {
                return Some(v);
            }
            Err(_) => {}
        }
    }
    None
}

fn split_by_line(code: String) -> Vec<(usize, String)> {
    let mut ret = vec![];

    for (l, content) in code.split_terminator('\n').enumerate() {
        ret.push((l, content.trim().to_string()));
    }

    ret
}
fn set_attribute(code: Vec<(usize, String)>) -> Vec<(usize, Attribute, String)> {
    let mut ret = vec![];

    for (l, content) in code {
        // if this line is comment, continue
        if content.starts_with("*") {
            continue;
        }
        match (to_instruction(&content), to_pseudo_instruction(&content)) {
            (Some(_), Some(_)) => {
                panic!();
            }
            (Some(v), None) => {
                ret.push((l, Attribute::Instruction(v), content));
            }
            (None, Some(v)) => {
                ret.push((l, Attribute::PseudoInstruction(v), content));
            }
            (None, None) => {
                panic!();
            }
        }
    }

    ret
}
fn get_addr(s: &str) -> Option<String> {
    let mut iter = s.split_whitespace().peekable();

    while iter.peek().is_some() {
        let slice = iter.next().unwrap();
        if Instruction::from_str(slice).is_ok() || PseudoInstruction::from_str(slice).is_ok() {
            // slice is instruction
            break;
        }
    }
    match iter.next() {
        None => None,
        Some(address) => Some(
            address
                .split(|c| c == '(' || c == ',')
                .collect::<Vec<&str>>()[0]
                .to_string(),
        ),
    }
}
fn get_index(s: &str) -> Option<String> {
    match s.find(",") {
        Some(mid) => Some(
            s.split_at(mid + 1).1.split("(").collect::<Vec<&str>>()[0]
                .trim()
                .to_string(),
        ),
        None => None,
    }
}
fn get_modification(s: &str) -> Option<String> {
    match s.find("(") {
        Some(mid) => Some(
            s.split_at(mid + 1).1.split(")").collect::<Vec<&str>>()[0]
                .trim()
                .to_string(),
        ),
        None => None,
    }
}
fn get_loc1(s: &str) -> Option<String> {
    let mut iter = s.split_whitespace();
    let head = iter.next().unwrap();
    match Instruction::from_str(head) {
        Ok(_) => None,
        Err(_) => Some(head.to_string()),
    }
}
fn get_loc2(s: &str) -> Option<String> {
    let mut iter = s.split_whitespace();
    let head = iter.next().unwrap();
    match PseudoInstruction::from_str(head) {
        Ok(_) => None,
        Err(_) => Some(head.to_string()),
    }
}
fn tokenize(code: Vec<(usize, Attribute, String)>) -> Vec<(usize, Option<String>, Content)> {
    let mut ret = vec![];

    for (line, attribute, content) in code {
        match attribute {
            Attribute::Instruction(v) => ret.push((
                line,
                get_loc1(&content),
                Content::Instruction(
                    v,
                    get_addr(&content),
                    get_index(&content),
                    get_modification(&content),
                ),
            )),
            Attribute::PseudoInstruction(PseudoInstruction::ALF) => ret.push((
                line,
                get_loc2(&content),
                Content::PseudoInstruction(PseudoInstruction::ALF, get_addr(&content), None),
            )),
            Attribute::PseudoInstruction(v) => ret.push((
                line,
                get_loc2(&content),
                Content::PseudoInstruction(v, get_addr(&content), get_modification(&content)),
            )),
        }
    }

    ret
}
fn resolve_symbol(
    mut code: Vec<(usize, Option<String>, Content)>,
) -> (usize, Vec<(usize, usize, Content)>) {
    // 1. resolve EQU
    // TODO: valueに式があれば解決する。
    let mut symbols = HashMap::new();
    for (_, loc, content) in &mut code {
        match content {
            Content::PseudoInstruction(PseudoInstruction::EQU, addr, _f) => {
                // EQUなのでsymbolsに突っ込む
                let variable = loc.as_ref().unwrap().clone().trim().to_string();
                let value = addr.clone().unwrap().trim().to_string();
                symbols.insert(variable, value);
            }
            Content::PseudoInstruction(PseudoInstruction::ORIG, addr, _f)
            | Content::PseudoInstruction(PseudoInstruction::CON, addr, _f)
            | Content::PseudoInstruction(PseudoInstruction::END, addr, _f) => {
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
            Content::Instruction(_, addr, index, loc) => {
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
            }
            _ => {
                //pass
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
    for (line, loc, content) in code {
        if loc.is_some() {
            symbols.push(loc.unwrap(), line, address);
        }
        match content {
            Content::PseudoInstruction(PseudoInstruction::ORIG, addr, _f) => {
                address = addr.unwrap().parse::<usize>().unwrap();
            }
            Content::PseudoInstruction(PseudoInstruction::CON, addr, f) => {
                code_with_address.push((
                    line,
                    address,
                    Content::PseudoInstruction(PseudoInstruction::CON, addr, f),
                ));
                address += 1;
            }
            Content::PseudoInstruction(PseudoInstruction::ALF, addr, f) => {
                code_with_address.push((
                    line,
                    address,
                    Content::PseudoInstruction(PseudoInstruction::ALF, addr, f),
                ));
                address += 1;
            }
            Content::PseudoInstruction(PseudoInstruction::END, addr, _f) => {
                entry_point = symbols.table.get(&addr.unwrap()).unwrap()[0].1;
            }
            // inst
            Content::Instruction(v, addr, index, modification) => {
                code_with_address.push((
                    line,
                    address,
                    Content::Instruction(v, addr, index, modification),
                ));
                address += 1;
            }
            _ => {}
        }
    }

    // let entry_point = symbols.table.get("START").unwrap()[0].1;
    let entry_point = entry_point;

    // 3. resolve address
    for (line, address, content) in &mut code_with_address {
        match content {
            Content::Instruction(_, addr, _, _) => {
                if addr.is_some() && addr.as_ref().unwrap().parse::<i64>().is_err() {
                    let res = symbols.resolve(addr.as_ref().unwrap(), *line, *address);
                    *addr = if res.is_some() {
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

    for (line, address, content) in code {
        match content {
            Content::PseudoInstruction(v, addr, _f) => {
                if v == PseudoInstruction::CON {
                    let x = addr.unwrap().parse::<i64>().unwrap();
                    ret.push((line, address, WordImpl::from_val(x)));
                } else if v == PseudoInstruction::ALF {
                    let mut v = vec![];
                    for c in addr.unwrap().replace("_", " ").chars() {
                        v.push(char_to_num(c).unwrap() as u32);
                    }
                    ret.push((line, address, WordImpl::from_seq(Sign::Positive, &v)));
                } else {
                    panic!()
                }
            }
            Content::Instruction(v, addr, index, modification) => {
                let (mut a, mut i, mut f, c) = instruction_data(&v);
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
                    // f = modification.unwrap().parse::<i64>().unwrap();
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
        }
    }

    (entry_point, ret)
}

pub fn assemble(code: String) -> (usize, Vec<(usize, usize, WordImpl)>) {
    encode_to_binary(resolve_symbol(tokenize(set_attribute(split_by_line(code)))))
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
    let tokenized_code = tokenize(set_attribute(split_by_line(code)));
    let mut loc_len = 0usize;
    let mut a_len = 0usize;
    let mut i_len = 0usize;
    let mut f_len = 0usize;
    for (_line, loc, content) in &tokenized_code {
        if loc.is_some() {
            loc_len = max(loc_len, loc.as_ref().unwrap().len());
        }
        match content {
            Content::Instruction(_inst, a, i, f) => {
                if a.is_some() {
                    a_len = max(a_len, a.as_ref().unwrap().len())
                }
                if i.is_some() {
                    i_len = max(i_len, i.as_ref().unwrap().len())
                }
                if f.is_some() {
                    f_len = max(f_len, f.as_ref().unwrap().len())
                }
            }
            Content::PseudoInstruction(_inst, a, f) => {
                if a.is_some() {
                    a_len = max(a_len, a.as_ref().unwrap().len())
                }
                if f.is_some() {
                    f_len = max(f_len, f.as_ref().unwrap().len())
                }
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
    for (line, loc, content) in tokenized_code {
        let li = format!("{:4}", line);
        let lo = if loc.is_some() {
            format_finite_length(&loc.unwrap(), loc_len)
        } else {
            " ".repeat(loc_len).to_string()
        };
        let op = match &content {
            Content::Instruction(inst, _a, _i, _f) => format_finite_length(&inst.to_string(), 6),
            Content::PseudoInstruction(inst, _a, _f) => format_finite_length(&inst.to_string(), 6),
        };
        let a = match &content {
            Content::Instruction(_inst, a, _i, _f) => {
                if a.is_some() {
                    format_finite_length(a.as_ref().unwrap(), a_len)
                } else {
                    " ".repeat(a_len).to_string()
                }
            }
            Content::PseudoInstruction(_inst, a, _f) => {
                if a.is_some() {
                    format_finite_length(a.as_ref().unwrap(), a_len)
                } else {
                    " ".repeat(a_len).to_string()
                }
            }
        };
        let i = match &content {
            Content::Instruction(_inst, _a, i, _f) => {
                if i.is_some() {
                    format_finite_length(i.as_ref().unwrap(), i_len)
                } else {
                    " ".repeat(i_len).to_string()
                }
            }
            Content::PseudoInstruction(_inst, _a, _f) => " ".repeat(i_len).to_string(),
        };
        let f = match content {
            Content::Instruction(_inst, _a, _i, f) => {
                if f.is_some() {
                    format_finite_length(f.as_ref().unwrap(), f_len)
                } else {
                    " ".repeat(f_len).to_string()
                }
            }
            Content::PseudoInstruction(_inst, _a, f) => {
                if f.is_some() {
                    format_finite_length(f.as_ref().unwrap(), f_len)
                } else {
                    " ".repeat(f_len).to_string()
                }
            }
        };
        ret.push_str(" | ");
        ret.push_str(&li);
        ret.push_str(" | ");
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
        let _tmp = encode_to_binary(resolve_symbol(tokenize(set_attribute(split_by_line(
            code.to_string(),
        )))));
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
        let _tmp = encode_to_binary(resolve_symbol(tokenize(set_attribute(split_by_line(
            code.to_string(),
        )))));
        let tmp = format_code(code.to_string());
        println!("\n{}", tmp);
    }
}
