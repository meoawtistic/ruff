use crate::error::Error;
use crate::value::{Literal, Value};

#[derive(Debug, Clone)]
pub enum Macro {
    Values(Vec<Value>),
    Macro {
        name: String,
        idx: usize,
        args: Vec<String>,
        content: Vec<Value>,
    },
    JumpTable {
        name: String,
        labels: Vec<String>,
    },
}

impl Macro {
    pub fn new(name: String, idx: usize, args: Vec<String>, content: Vec<Value>) -> Self {
        Macro::Macro {
            name,
            content,
            idx,
            args: if args.len() == 1 && args[0].is_empty() {
                vec![]
            } else {
                args
            },
        }
    }
}

pub fn push_n(n: usize) -> Result<Literal, Error> {
    if n < 1 || n > 32 {
        return Err(Error::new("cant push value: bad size"));
    };
    Ok(Literal::from(95 + n))
}
