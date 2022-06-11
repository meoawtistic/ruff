use std::collections::HashMap;

use crate::macros::Macro;
use crate::value::Value;

#[derive(Debug)]
pub struct Context {
    pub jumptables: Vec<(String, Vec<String>, Option<usize>)>, // (name, [labels], start)
    pub jumptabledests: HashMap<String, usize>,
    pub jumptablestarts: HashMap<String, Vec<usize>>,
    pub macros: HashMap<String, Macro>,
    pub constants: HashMap<String, Value>,
    pub compiles_at_idx: HashMap<(String, usize), usize>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            jumptables: Vec::new(),
            jumptabledests: HashMap::new(),
            jumptablestarts: HashMap::new(),
            macros: HashMap::new(),
            constants: HashMap::new(),
            compiles_at_idx: HashMap::new(),
        }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for c in self.constants.iter() {
            s.push_str(format!("constant  {} {:?} \n", c.0, c.1).as_str());
        }
        for c in self.macros.iter() {
            s.push_str(format!("macro     {} {:?} \n", c.0, c.1).as_str());
        }
        s
    }

    pub fn exists(&self, s: &str) -> bool {
        let s = s.trim();

        self.constants.get(s).is_some()
            || self.macros.get(s).is_some()
            || self.jumptables.iter().find(|j| &j.0 == s).is_some()
    }

    pub fn resolve_constant(&self, s: &str) -> Option<Value> {
        self.constants.get(s.trim()).map(|c| c.clone())
    }

    pub fn resolve_macro(&self, s: &str) -> Option<Macro> {
        self.macros.get(s.trim()).map(|x| x.clone())
    }

    pub fn resolve_jumptabledest(&self, s: &str) -> Option<usize> {
        self.jumptabledests.get(s.trim()).map(|x| x.clone())
    }
}
