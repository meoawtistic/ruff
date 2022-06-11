use std::fmt;
use std::fmt::format;

use ethers::types::{Bytes, U256};
use hex::decode;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Literal {
    value: Bytes,
}

impl Literal {
    pub fn new(s: &str) -> Result<Self, Error> {
        let s = s.replacen("_", "", usize::MAX);
        if s.starts_with("0x") {
            if let Ok(bytes) = hex::decode(&s[2..]) {
                return match bytes.len() {
                    l if l <= 32 => Ok(Literal {
                        value: Bytes::from(bytes),
                    }),
                    _ => Err(Error::new(&format!("literal `{}` overflows uint256", s))),
                };
            }
        }

        let v = U256::from_dec_str(s.as_str())?;
        Ok(Literal { value: u2b(&v) })
    }

    pub fn to_string(&self) -> String {
        format!("{:x}", self.value)
    }

    pub fn to_hex(&self) -> String {
        self.value
            .to_vec()
            .iter()
            .map(|i| match format!("{:x}", i) {
                s if s.len() % 2 == 0 => s,
                s => "0".to_string() + &s,
            })
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn from(s: usize) -> Literal {
        Literal {
            value: u2b(&U256::from(s)), //
        }
    }

    pub fn from_i64(i: i64) -> Literal {
        Literal::from(i as usize)
    }
}

#[test]
fn test_literal() {
    let a = Literal::new("0x0c80");
    let a = a.unwrap().value;
    assert_eq!(a.to_vec().len(), 2);
    assert_eq!(U256::from(3200 as usize), b2u(&a));

    let a = Literal::new("0x0000000001");
    let a = a.unwrap();
    assert_eq!(a.value.to_vec().len(), 5);
    assert_eq!(a.to_hex(), "0000000001");

    let a = Literal::new("03200");
    let a = a.unwrap();
    assert_eq!(a.value, u2b(&U256::from("0xc80")));
    assert_eq!(a.to_hex(), "0c80");
}

// todo: add unknown value type?  currently label acts as this

#[derive(Debug, Clone)]
pub enum Value {
    Label {
        name: String,
        ns: Option<String>,
    },
    JumpDest {
        name: String,
    },
    JumpTableDest {
        name: String,
    },
    Constant {
        name: String,
        value: Box<Value>,
    },
    Opcode(u8),
    Literal(Literal),
    Quote(Box<Value>),
    MathOp {
        op: MathOps,
        left: Box<Value>,
        right: Box<Value>,
    },
    SizeCall(SizeCall),
    StartCall(JumpTableCall),
    MacroCall(MacroCall), // Todo: expands into multiple values, handle differently
    JumpTableCall(JumpTableCall), // Todo: just sits inside size() and start(), handle differently
}

#[derive(Debug, Clone)]
pub struct JumpTableCall {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MacroCall {
    // todo: not actually a value but needs to sit inside a value, macros might need to change too?
    //          Or create an enum { value, valueArr }
    pub name: String,
    pub args: Option<Vec<Value>>,
}

fn b2u(b: &Bytes) -> U256 {
    U256::from(&b.to_vec()[..])
}
fn u2b(u: &U256) -> Bytes {
    let b = if u.is_zero() { 1 } else { (u.bits() + 7) / 8 };
    Bytes::from(
        (0..b)
            .into_iter()
            .enumerate()
            .rev()
            .map(|(i, ..)| u.byte(i))
            .collect::<Vec<u8>>(),
    )
}

#[derive(Debug, Clone)]
pub enum SizeCall {
    Macro(MacroCall),
    JumpTable { name: String },
}

#[derive(Debug, Clone)]
pub enum MathOps {
    Add,
    Sub,
    Mul,
    Div,
}

impl MathOps {
    pub fn to_string(&self) -> String {
        match self {
            MathOps::Add => "add".to_string(),
            MathOps::Sub => "sub".to_string(),
            MathOps::Mul => "mul".to_string(),
            MathOps::Div => "div".to_string(),
        }
    }

    pub fn to_op(&self) -> String {
        match self {
            MathOps::Add => "+".to_string(),
            MathOps::Sub => "-".to_string(),
            MathOps::Mul => "*".to_string(),
            MathOps::Div => "/".to_string(),
        }
    }

    pub fn all() -> Vec<MathOps> {
        vec![MathOps::Add, MathOps::Sub, MathOps::Mul, MathOps::Div]
    }

    pub fn all_ops() -> Vec<String> {
        MathOps::all().iter().map(|o| o.to_op()).collect()
    }
}

impl Value {
    pub fn new_literal(s: &str) -> Result<Self, Error> {
        Ok(Value::Literal(Literal::new(s)?))
    }
}

pub fn apply(op: &MathOps, left: &Literal, right: &Literal) -> Result<Literal, Error> {
    match match op {
        MathOps::Add => b2u(&left.value).checked_add(b2u(&right.value)),
        MathOps::Sub => b2u(&left.value).checked_sub(b2u(&right.value)),
        MathOps::Mul => b2u(&left.value).checked_mul(b2u(&right.value)),
        MathOps::Div => b2u(&left.value).checked_div(b2u(&right.value)),
    } {
        None => Err(Error::new(&(op.to_string() + " overflow"))),
        Some(ref value) => Ok(Literal { value: u2b(value) }),
    }
}

#[test]
fn test_apply_add() {
    let (a, b) = (Literal::new("0x01").unwrap(), Literal::new("03").unwrap());
    let sum = apply(&MathOps::Add, &a, &b);
    assert!(sum.is_ok());
    assert_eq!(
        sum.unwrap().to_string(),
        Literal::new("0_4").unwrap().to_string()
    );
}
