use crate::common::{instruction_data, Instruction, CHAR_TABLE};
use crate::mix_word::{Byte, Sign, WordImpl};
use std::collections::HashMap;
use std::str::FromStr;

const OPECODE: [&'static str; 145] = [
    "NOP", "ADD", "SUB", "MUL", "DIV", "NUM", "CHAR", "HLT", "SLA", "SRA", "SLAX", "SRAX", "SLC",
    "SRC", "MOVE", "LDA", "LD1", "LD2", "LD3", "LD4", "LD5", "LD6", "LDX", "LDAN", "LD1N", "LD2N",
    "LD3N", "LD4N", "LD5N", "LD6N", "LDXN", "STA", "ST1", "ST2", "ST3", "ST4", "ST5", "ST6", "STX",
    "STJ", "STZ", "JBUS", "IOC", "IN", "OUT", "JRED", "JMP", "JSJ", "JOV", "JNOV", "JL", "JE",
    "JG", "JGE", "JNE", "JLE", "JAN", "JAZ", "JAP", "JANN", "JANZ", "JANP", "J1N", "J1Z", "J1P",
    "J1NN", "J1NZ", "J1NP", "J2N", "J2Z", "J2P", "J2NN", "J2NZ", "J2NP", "J3N", "J3Z", "J3P",
    "J3NN", "J3NZ", "J3NP", "J4N", "J4Z", "J4P", "J4NN", "J4NZ", "J4NP", "J5N", "J5Z", "J5P",
    "J5NN", "J5NZ", "J5NP", "J6N", "J6Z", "J6P", "J6NN", "J6NZ", "J6NP", "JXN", "JXZ", "JXP",
    "JXNN", "JXNZ", "JXNP", "INCA", "DECA", "ENTA", "ENNA", "INC1", "DEC1", "ENT1", "ENN1", "INC2",
    "DEC2", "ENT2", "ENN2", "INC3", "DEC3", "ENT3", "ENN3", "INC4", "DEC4", "ENT4", "ENN4", "INC5",
    "DEC5", "ENT5", "ENN5", "INC6", "DEC6", "ENT6", "ENN6", "INCX", "DECX", "ENTX", "ENNX", "CMPA",
    "FCMP", "CMP1", "CMP2", "CMP3", "CMP4", "CMP5", "CMP6", "CMPX",
];
const KEYWORD: [&'static str; 5] = ["EQU", "ORIG", "CON", "ALF", "END"];

#[derive(Debug)]
enum Attribute {
    Instruction(usize),
    Support(usize),
}
#[derive(Debug)]
enum Content {
    Support(usize, Option<String>),
    Instruction(usize, Option<String>, Option<String>, Option<String>),
}

fn operation(s: &str) -> Option<usize> {
    for x in 0..OPECODE.len() {
        for slice in s.split_whitespace() {
            if slice == OPECODE[x] {
                return Some(x);
            }
        }
    }
    None
}
fn keyword(s: &str) -> Option<usize> {
    for x in 0..KEYWORD.len() {
        for slice in s.split_whitespace() {
            if slice == KEYWORD[x] {
                return Some(x);
            }
        }
    }
    None
}

pub fn split_by_line(code: String) -> Vec<(usize, String)> {
    let mut ret = vec![];

    for (l, content) in code.split_terminator('\n').enumerate() {
        ret.push((l, content.trim().to_string()));
    }

    ret
}
fn set_attribute(code: Vec<(usize, String)>) -> Vec<(usize, Attribute, String)> {
    let mut ret = vec![];

    for (l, content) in code {
        match operation(&content) {
            Some(v) => {
                ret.push((l, Attribute::Instruction(v), content));
            }
            None => match keyword(&content) {
                Some(v) => {
                    ret.push((l, Attribute::Support(v), content));
                }
                None => {
                    if !content.starts_with("*") {
                        println!("comment should be started with *.\n{:?}\n^^^", content);
                    }
                }
            },
        }
    }

    ret
}
fn get_addr(s: &str) -> Option<String> {
    let mut expect_address = false;

    for slice in s.split_whitespace() {
        match operation(&slice) {
            Some(_) => expect_address = true,
            None => {
                if expect_address {
                    // 空白 , or ( が始まるまではaddrのトークン
                    return Some(
                        slice.split("(").collect::<Vec<&str>>()[0]
                            .split(",")
                            .collect::<Vec<&str>>()[0]
                            .to_string(),
                    );
                }
            }
        }
    }

    None
}
fn get_index(s: &str) -> Option<String> {
    match s.find(",") {
        Some(_) => Some(
            s.split(",").collect::<Vec<&str>>()[1]
                .split("(")
                .collect::<Vec<&str>>()[0]
                .trim()
                .to_string(),
        ),
        None => None,
    }
}
fn get_modification(s: &str) -> Option<String> {
    match s.find("(") {
        Some(_) => Some(
            s.split("(").collect::<Vec<&str>>()[1]
                .split(")")
                .collect::<Vec<&str>>()[0]
                .trim()
                .to_string(),
        ),
        None => None,
    }
}
fn get_loc1(s: &str) -> Option<String> {
    for slice in s.split_whitespace() {
        match operation(slice) {
            Some(_) => break,
            None => return Some(slice.to_string()),
        }
    }
    None
}
fn get_loc2(s: &str) -> (Option<String>, String) {
    let mut op = 0;
    let mut op_pos = 0;
    for (i, slice) in s.split_whitespace().enumerate() {
        match keyword(slice) {
            Some(v) => {
                op = v;
                op_pos = i;
            }
            None => {}
        }
    }
    if (op == 2 || op == 3) && op_pos == 1 {
        let mid = s.find(' ').unwrap();
        let (x1, x2) = s.split_at(mid);
        (Some(x1.trim().to_string()), x2.trim().to_string())
    } else {
        (None, s.to_string())
    }
}
fn parse_content(code: Vec<(usize, Attribute, String)>) -> Vec<(usize, Option<String>, Content)> {
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
            Attribute::Support(v) => {
                let (loc, content) = get_loc2(&content);
                match content.trim().find(' ') {
                    Some(keyword_end) => {
                        if v == 1 || v == 2 || v == 3 {
                            let (_keyword, content) = content.split_at(keyword_end);
                            ret.push((
                                line,
                                loc,
                                Content::Support(v, Some(content.trim().to_string())),
                            ));
                        } else {
                            ret.push((
                                line,
                                loc,
                                Content::Support(v, Some(content.trim().to_string())),
                            ));
                        }
                    }
                    None => {
                        ret.push((line, loc, Content::Support(v, None)));
                    }
                }
            }
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
    for (_, _, content) in &mut code {
        match content {
            Content::Support(0, s) => {
                // EQUなのでsymbolsに突っ込む
                let mut s = s.clone().unwrap();
                let equ_pos = s.find("EQU").unwrap();
                let variable = s.get_mut(..equ_pos).unwrap().trim().to_string();
                let value = s.get_mut((equ_pos + 3)..).unwrap().trim().to_string();
                symbols.insert(variable, value);
            }
            Content::Support(1, s) | Content::Support(2, s) | Content::Support(3, s) => {
                if s.is_some() {
                    let tmp_s = s.clone().unwrap();
                    match symbols.get(&tmp_s) {
                        Some(value) => {
                            *s = Some(value.to_string());
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
    // for (line, loc, content) in &code {
    //     println!("line: {}, loc: {:?}, content: {:?}", line, loc, content);
    // }

    // 2. construct symbol table
    let mut symbols = SymbolTable {
        table: HashMap::new(),
    };
    let mut code_with_address = vec![];
    let mut address = 0usize;
    for (line, loc, content) in code {
        if loc.is_some() {
            symbols.push(loc.unwrap(), line, address);
        }
        match content {
            // orig
            Content::Support(1, s) => {
                address = s.unwrap().parse::<usize>().unwrap();
            }
            // CON
            Content::Support(2, s) => {
                code_with_address.push((line, address, Content::Support(2, s)));
                address += 1;
            }
            // ALF
            Content::Support(3, s) => {
                code_with_address.push((line, address, Content::Support(3, s)));
                address += 1;
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
    // for (line, address, content) in &code_with_address {
    //     println!(
    //         "line: {:2}, address: {:4?}, content: {:?}",
    //         line, address, content
    //     );
    // }
    let entry_point = symbols.table.get("START").unwrap()[0].1;
    // println!("Entry Point{}\nSYMBOL: {:?}", entry_point, symbols);

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
    // for (line, address, content) in &code_with_address {
    //     println!(
    //         "line: {:2}, address: {:4?}, content: {:?}",
    //         line, address, content
    //     );
    // }

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
            Content::Support(v, s) => {
                if v == 2 {
                    // CON
                    let x = s.unwrap().parse::<i64>().unwrap();
                    ret.push((line, address, WordImpl::from_val(x)));
                } else {
                    // ALF
                    let mut v = vec![];
                    for c in s.unwrap().replace("_", " ").chars() {
                        v.push(char_to_num(c).unwrap() as u32);
                    }
                    ret.push((line, address, WordImpl::from_seq(Sign::Positive, &v)));
                }
            }
            Content::Instruction(v, addr, index, modification) => {
                let (mut a, mut i, mut f, c) =
                    instruction_data(&Instruction::from_str(&OPECODE[v]).unwrap());
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
    // for (line, address, word) in &ret {
    //     println!(
    //         "[binary] line: {:2}, address: {:4}, word: {}",
    //         line, address, word
    //     );
    // }

    (entry_point, ret)
}

pub fn assemble(code: String) -> (usize, Vec<(usize, usize, WordImpl)>) {
    encode_to_binary(resolve_symbol(parse_content(set_attribute(split_by_line(
        code,
    )))))
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
        let _tmp = encode_to_binary(resolve_symbol(parse_content(set_attribute(split_by_line(
            code.to_string(),
        )))));
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
                    EXIT JMP *";
        let _tmp = encode_to_binary(resolve_symbol(parse_content(set_attribute(split_by_line(
            code.to_string(),
        )))));
    }
}
