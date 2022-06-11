use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Formatter;

use ethers::types::U256;
use evalexpr::{build_operator_tree, Node, Operator};
use regex::Regex;

use crate::context::Context;
use crate::error::Error;
use crate::macros::Macro;
use crate::opcodes::OPCODES;
use crate::value::{JumpTableCall, Literal, MacroCall, MathOps, SizeCall, Value};

pub fn remove_comments(s: &str) -> Result<String, Error> {
    let mut data = s;
    let mut formatted = String::new();
    while !data.trim().is_empty() {
        let idx_multi = data.find("/*");
        let idx_single = data.find("//");
        if idx_multi.is_some() && (idx_single.is_none() || idx_multi < idx_single) {
            formatted.push_str(&data[0..idx_multi.unwrap()]);
            match data.find("*/") {
                None => return Err(Error::new("missing close of multiline comment")),
                Some(end) => {
                    formatted.push_str(&" ".repeat(end - idx_multi.unwrap() + 2));
                    data = &data[end + 2..];
                }
            }
        } else if let Some(idx_single) = idx_single {
            formatted.push_str(&data[0..idx_single]);
            data = &data[idx_single..];

            match data.find("\n") {
                None => break,
                Some(0) => {
                    formatted.push_str(&" ".repeat(data.len()));
                    break;
                }
                Some(end) => {
                    formatted.push_str(&" ".repeat(end));
                    data = &data[end..];
                }
            }
        } else {
            formatted.push_str(data);
            break;
        }
    }

    Ok(formatted)
}

fn is_label_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

// requires single line input
pub fn parse_value(s: &str, c: &Context) -> Result<Value, Error> {
    let s = s.trim();
    if s.is_empty() {
        return Err(Error::new("empty value"));
    }

    if MathOps::all_ops().iter().all(|o| !s.contains(o)) {
        if let Ok(l) = Literal::new(s) {
            return Ok(Value::Literal(l));
        }

        if let Some(o) = OPCODES.get(s) {
            match U256::from_str_radix(o, 16) {
                Err(e) => panic!("{} {} {}", s, o, e.to_string()),
                Ok(o) => return Ok(Value::Opcode(o.as_u32() as u8)),
            }
        }

        if s.ends_with(":") && s[..s.len() - 1].chars().all(is_label_char) {
            return Ok(Value::JumpDest {
                name: s[..s.len() - 1].to_string(),
            });
        }

        if s.len() > 2
            && s.starts_with(":")
            && s.ends_with(":")
            && s[1..s.len() - 1].chars().all(is_label_char)
        {
            return Ok(Value::JumpTableDest {
                name: s[1..s.len() - 1].to_string(),
            });
        }

        if s.chars().all(is_label_char) {
            return Ok(Value::Label {
                name: s[..s.len()].to_string(),
                ns: None,
            });
        }

        if s.len() > 2
            && s.chars().nth(0).unwrap() == '"'
            && s.chars().nth(s.len() - 1).unwrap() == '"'
        {
            let inside = &s[1..s.len() - 1];

            match parse_expression(inside, c) {
                Ok(m) => match macro_to_value(&m) {
                    Ok(v) => {
                        return Ok(Value::Quote(Box::new(v)));
                    }
                    Err(e) => {
                        // println!("parsing between quotes {}", e)
                    }
                },
                Err(e) => {
                    // println!("parsing between quotes {}", e)
                }
            }
        }
        if let Ok(v) = parse_sizecall(s, c) {
            return Ok(v);
        };

        if let Ok(v) = parse_startcall(s, c) {
            return Ok(v);
        };
    } else {
        return Ok(parse_math(s, c)?);
    }

    Err(Error::new("unimpl."))
}

fn parse_sizecall(s: &str, c: &Context) -> Result<Value, Error> {
    if rgx(Parsable::SizeCall).is_match(s) {
        if let Some(m) = rgx(Parsable::SizeCall).captures(s) {
            let content = m.get(1).unwrap().as_str();

            if let Ok(mc) = parse_macro_call(content, c) {
                return Ok(Value::SizeCall(SizeCall::Macro(mc)));
            }

            return Ok(Value::SizeCall(SizeCall::JumpTable {
                name: content.to_string(),
            }));
        }
    }

    Err(Error::new("failed to parse size call"))
}

fn parse_startcall(s: &str, c: &Context) -> Result<Value, Error> {
    if rgx(Parsable::StartCall).is_match(s) {
        if let Some(m) = rgx(Parsable::StartCall).captures(s) {
            return Ok(Value::StartCall(JumpTableCall {
                name: m.get(1).unwrap().as_str().to_string(),
            }));
        }
    }

    Err(Error::new("failed to parse start call"))
}

fn parse_math(s: &str, c: &Context) -> Result<Value, Error> {
    let r = build_operator_tree(s)?;

    fn node_to_value(n: &Node, c: &Context) -> Result<Value, Error> {
        let children = n.children();

        match n.operator() {
            Operator::Add | Operator::Div | Operator::Mul | Operator::Sub => {
                if children.len() != 2 {
                    return Err(Error::new(&format!(
                        "operator {} takes 2 arguments",
                        n.operator().to_string()
                    )));
                }

                let op = match n.operator() {
                    Operator::Add => MathOps::Add,
                    Operator::Div => MathOps::Div,
                    Operator::Mul => MathOps::Mul,
                    Operator::Sub => MathOps::Sub,
                    _ => panic!(""),
                };

                Ok(Value::MathOp {
                    op,
                    left: Box::new(node_to_value(&children[0], c)?),
                    right: Box::new(node_to_value(&children[1], c)?),
                })
            }

            Operator::RootNode => return node_to_value(&n.children()[0], c),
            Operator::VariableIdentifier { identifier } => {
                let pm = parse_expression(identifier, c);
                if let Err(e) = pm {
                    return Err(e);
                }

                return Ok(macro_to_value(&pm.unwrap())?);
            }
            Operator::Const { value } => match value {
                evalexpr::Value::Int(i) => Ok(Value::Literal(Literal::from_i64(*i))),
                _ => {
                    return Err(Error::new(&format!("const not supported {:?}", value)));
                }
            },
            Operator::FunctionIdentifier { identifier } => {
                let children = children[0].children();
                let children = if children.len() == 1 {
                    let op = children[0].operator().clone();
                    if op == Operator::Tuple {
                        Some(children[0].children())
                    } else if op == Operator::RootNode {
                        None
                    } else {
                        Some(children)
                    }
                } else {
                    Some(children)
                };

                match identifier.as_str() {
                    "size" => {
                        if children.is_none() || children.unwrap().len() != 1 {
                            return Err(Error::new("size call takes one argument"));
                        }

                        match node_to_value(&children.unwrap()[0], c) {
                            Ok(Value::MacroCall(m)) => Ok(Value::MacroCall(m)),
                            // TODO: support jump tables
                            Err(e) => Err(e),
                            _ => Err(Error::new("only macro call allowed in size")),
                        }
                    }
                    "start" => panic!("start function not impl yet"),
                    i => {
                        let args = if children.is_some() {
                            Some(
                                children
                                    .unwrap()
                                    .iter()
                                    .map(|n| node_to_value(n, c))
                                    .collect::<Result<Vec<Value>, Error>>()?,
                            )
                        } else {
                            None
                        };

                        return Ok(Value::MacroCall(MacroCall {
                            name: i.to_string(),
                            args,
                        }));
                    }
                }
            }
            o => {
                return Err(Error::new(&format!(
                    "operator not supported {:?} in node:  {:?}",
                    o, n
                )))
            }
        }
    }

    Ok(node_to_value(&r, c)?)
}

pub fn parse_macro(
    name: &str,
    idx: usize,
    args: &str,
    content: &str,
    c: &Context,
) -> Result<Macro, Error> {
    let args = args
        .trim()
        .split(",")
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    let mut content = content
        .trim()
        .split_whitespace()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    let mut i = 0;
    while i < content.len() {
        match find_math(&content, i) {
            Ok(end) => {
                if i != end {
                    let math = vec![content[i..end + 1].join("")];

                    content = if end == content.len() - 1 {
                        if i == 0 {
                            [&math[..]].concat()
                        } else {
                            [&content[..i], &math[..]].concat()
                        }
                    } else {
                        if i == 0 {
                            [&math[..], &content[end + 1..]].concat()
                        } else {
                            [&content[..i], &math[..], &content[end + 1..]].concat()
                        }
                    };
                }
            }
            Err(e) => return Err(e),
        }
        i += 1;
    }
    // TODO: handle math in macro here

    let content = content
        .iter()
        .map(|s| macro_to_value(&parse_expression(s.trim(), c)?))
        .collect::<Result<Vec<Value>, Error>>()?;

    let mut jumpdests = HashMap::<String, ()>::new();

    for v in content.iter() {
        if let Value::JumpDest { name } = v {
            if args.contains(name) {
                return Err(Error::new(&format!(
                    "jumpdest label `{}` shares name with argument",
                    name
                )));
            }

            if jumpdests.contains_key(name) {
                return Err(Error::new(&format!("jumpdest label `{}` reused", name)));
            }

            jumpdests.insert(name.to_string(), ());
        }
    }

    let mut symbol_not_found: Option<String> = None;

    let mut content = content;
    let content: Vec<Option<Value>> = content
        .iter_mut()
        .map(|v| {
            if let Value::Label { name, .. } = v {
                if !(args.contains(name) || jumpdests.contains_key(name)) {
                    return if let Some(new_v) = c.resolve_constant(name) {
                        Some(new_v)
                    } else {
                        symbol_not_found = Some(name.to_string());
                        None
                    };
                }
            }
            Some(v.clone())
        })
        .collect::<Vec<Option<Value>>>();

    if let Some(s) = symbol_not_found {
        return Err(Error::new(&format!(
            "`{}` not defined in args or as jumpdest in macro `{}` body",
            s, name
        )));
    }

    let content = content.iter().map(|v| v.clone().unwrap()).collect();

    if name == "size" || name == "start" {
        return Err(Error::new(&format!(
            "cant use reserved keyword `{}` as macro name",
            name
        )));
    }

    Ok(Macro::new(name.to_string(), idx, args, content))
}

fn parse_macro_call(s: &str, c: &Context) -> Result<MacroCall, Error> {
    let ca = match rgx(Parsable::MacroCall).captures(s) {
        None => return Err(Error::new(&format!("bad macro call @ `{}`", s))),
        Some(c) => c,
    };

    let name = ca.get(1).unwrap().as_str().to_string();
    let args = match ca.get(2).map(|m| split_call_args(m.as_str())) {
        Some(a) => Some(
            a?.iter()
                .map(|x| macro_to_value(&parse_expression(x, c)?))
                .collect::<Result<Vec<Value>, Error>>()?,
        ),
        None => None,
    };

    Ok(MacroCall { name, args })
}

// todo: for params in macro call
fn split_call_args(s: &str) -> Result<Vec<String>, Error> {
    let s = s.trim();
    if s.len() == 0 {
        return Ok(vec![]);
    }

    if !s.contains("(") && !s.contains(")") {
        return Ok(s.split(",").map(|s| s.trim().to_string()).collect());
    }

    // TODO: handle parens:

    return Err(Error::new("not impl ( in args"));
}

// TODO   is this always called in a macro? should it be macrocontext?
// todo   for usage  inside  parse macro call:
//        move resolve_value into  parse_value  & use latter
pub fn parse_expression(s: &str, c: &Context) -> Result<Macro, Error> {
    if let Some(v) = c.resolve_macro(s) {
        return Ok(v);
    }

    if let Some(v) = c.resolve_constant(s) {
        return Ok(Macro::Values(vec![v]));
    }

    if let Ok(v) = parse_value(s, c) {
        return Ok(Macro::Values(vec![v]));
    }

    if s.contains("(") && s.contains(")") {
        match parse_macro_call(s, c) {
            Ok(v) => Ok(Macro::Values(vec![Value::MacroCall(v)])),
            Err(e) => Err(e),
        }
    } else {
        match c.jumptables.iter().find(|j| j.0 == s) {
            Some((..)) => Ok(Macro::Values(vec![Value::JumpTableCall(JumpTableCall {
                name: s.to_string(),
            })])),
            None => Err(Error::new(&format!("not defined: `{}`", s))),
        }
    }
}

// to downcast, not to evaluate a macro
fn macro_to_value(m: &Macro) -> Result<Value, Error> {
    match m {
        Macro::Macro { .. } | Macro::JumpTable { .. } => {
            Err(Error::new("expected value not macro"))
        }
        Macro::Values(v) => {
            if v.len() == 1 {
                Ok(v[0].clone())
            } else {
                Err(Error::new("expected one value"))
            }
        }
    }
}

pub fn parse_top_level(s: &str) -> Result<Context, Error> {
    let mut idx = 0;

    let mut c = Context::new();
    let mut q = vec![];
    let mut loops = 0;
    let mut pending_error: Option<Error> = None;

    while idx < s.len() || !q.is_empty() {
        if !q.is_empty() && idx == s.len() {
            idx = q.pop().unwrap();
            loops += 1;
        } else if loops > q.len() + 1 {
            if let Some(e) = pending_error {
                return Err(e);
            }
            panic!("unhandled error")
        }

        if rgx(Parsable::Macro).is_match(&s[idx..]) {
            // TODO: if already parsed, skip

            let f = rgx(Parsable::Macro).captures(&s[idx..]).unwrap();
            let name = f.get(1).unwrap().as_str();
            let args = f.get(2).unwrap().as_str();
            let content = f.get(3).unwrap();

            if let Some(Macro::Macro { idx: m_idx, .. }) = c.macros.get(name) {
                if idx != *m_idx {
                    return Err(Error::new(&format!("macro `{}` redefined", name)));
                }
            }

            let mac = parse_macro(name, idx, args, content.as_str(), &c);

            match mac {
                Err(e) => {
                    q.push(idx);
                    idx += content.end() + 1;
                    pending_error = Some(e);
                    continue;
                }
                Ok(m) => {
                    c.macros.insert(name.to_string(), m);

                    idx += content.end() + 1;
                }
            }
        } else if rgx(Parsable::JumpTable).is_match(&s[idx..]) {
            let f = rgx(Parsable::JumpTable).captures(&s[idx..]).unwrap();
            let name = f.get(1).unwrap();
            let content = f.get(2).unwrap();
            idx += content.end() + 1;

            c.jumptables.push((
                name.as_str().to_string(),
                content
                    .as_str()
                    .trim()
                    .split_whitespace()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>(),
                None,
            ));
        } else if rgx(Parsable::Constant).is_match(&s[idx..]) {
            let f = rgx(Parsable::Constant).captures(&s[idx..]).unwrap();

            let name = f.get(1).unwrap();
            let content = f.get(2).unwrap();
            idx += content.end();

            let val = parse_value(content.as_str(), &c);

            if let Err(e) = val {
                q.push(idx);
                idx += content.end() + 1;
                pending_error = Some(e);
                continue;
            }

            c.constants.insert(name.as_str().to_string(), val.unwrap());
        } else if rgx(Parsable::WhiteSpace).is_match(&s[idx..]) {
            let f = rgx(Parsable::WhiteSpace).captures(&s[idx..]).unwrap();
            let end = f.get(0).unwrap().end();
            idx += end;
        } else {
            return Err(Error::new(&format!("unexpected {}", &s[idx..])));
        }
    }
    Ok(c)
} //

fn find_math(s: &Vec<String>, start: usize) -> Result<usize, Error> {
    let is_op = |x| MathOps::all_ops().iter().any(|y| y == x);
    let mut i = start;
    let mut start = 0;
    let mut count = 0;

    let mut contains_parens = false;

    'outer: while i < s.len() {
        if contains_parens || s[i].contains("(") || s[i].contains(")") {
            contains_parens = true;
            'inner: for c in s[i].chars() {
                match c {
                    '(' => {
                        count += 1;
                        if count == 1 {
                            start = i;
                            continue 'inner;
                        }
                    }
                    ')' => {
                        count -= 1;
                        if count == 0 {
                            break 'outer;
                        }
                        continue 'inner;
                    }
                    _ => {}
                };
            }
        }

        if !contains_parens
            && (!is_op(&s[i]) && (i == s.len() - 1 || i < s.len() - 1 && !is_op(&s[i + 1])))
        {
            return Ok(i);
        }
        i += 1;
    }

    if count != 0 {
        return Err(Error::new("unclosed parenthesis"));
    }
    // todo: handle parens

    return Ok(i);
}

#[test]
fn test_find_math() {
    let cases = vec![
        ("1 + 4", 0, 2),
        ("100 / ( 4-2)", 0, 3),
        ("macro1(100 / ( 4-2))", 0, 3),
        ("macro1(100 / ( 4 -2))", 0, 4),
        ("macro1(100 / ( 4 - 2))", 0, 5),
        ("macro1(100 / ( 4 - 2 ))", 0, 6),
        ("macro1(100 / ( 4 - 2 ) )", 0, 7),
        ("macro1(100/ ( 4 - 2 ) )", 0, 6),
        ("macro1(100/( 4 - 2 ) )", 0, 5),
        ("macro1(100/(4 - 2 ) )", 0, 4),
    ];

    for (case_i, case) in cases
        .iter()
        .map(|c| {
            (
                c.0.split_whitespace().map(|x| x.to_string()).collect(),
                c.1,
                c.2,
            )
        })
        .collect::<Vec<(Vec<String>, usize, usize)>>()
        .iter()
        .enumerate()
    {
        match find_math(&case.0, case.1) {
            Err(e) => panic!("error {}", e),
            Ok(m) => {
                assert_eq!(m, case.2);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Num {
    s: String,
}

impl std::fmt::Display for Num {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.s)
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
enum Parsable {
    Macro,
    Constant,
    SizeCall,
    StartCall,
    MacroCall,
    JumpTable,
    WhiteSpace,
}

fn rgx(p: Parsable) -> Regex {
    Regex::new(match p {
        Parsable::Macro => r"^\s*(\w+)\s*=\s*\((.*)\)\s*\{\s*([^}]*)\s*}",
        Parsable::Constant => r"^\s*(\w+)\s*=\s*([^=\n]*)",
        Parsable::SizeCall => r"^\s*size\s*\(\s*(.+)\s*\)",
        Parsable::StartCall => r"^\s*start\s*\(\s*(.+)\s*\)",
        Parsable::MacroCall => r"^\s*(\w+)\(\s*(.*)\s*\)",
        Parsable::JumpTable => r"^\s*(\w+)\s*=\s*\{\s*([\w\s]*)\s*\}",
        Parsable::WhiteSpace => r"^\s+",
    })
    .unwrap()
}

#[test]
fn test_reg() {
    assert!(rgx(Parsable::Constant).is_match("asdf=asdf"));
    assert!(rgx(Parsable::Constant).is_match("asdf   = asdf   "));
    assert!(rgx(Parsable::Constant).is_match("9as_sf = asdf"));
    assert!(rgx(Parsable::Constant).is_match("asdf \n=  \nasdf"));
}

// TODO: test overflow of 1000000000000000000000000000000000000000000000000000000000000000000000000000
// TODO: test negative numbers
