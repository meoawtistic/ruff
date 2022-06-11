use evalexpr::EvalexprError;
use std::fmt::{Display, Formatter};
use std::ops::Add;

use uint::{FromDecStrErr, FromStrRadixErr};

#[derive(Debug)]
pub struct Error {
    msg: String,
    line: Option<usize>,
    col: Option<usize>,
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Error {
            msg: msg.to_string(),
            line: None,
            col: None,
        }
    }

    pub fn with_loc(self, line: usize, col: usize) -> Self {
        Error {
            msg: self.msg,
            line: Some(line),
            col: Some(col),
        }
    }

    pub fn set_loc(&mut self, line: usize, col: usize) {
        self.line = Some(line);
        self.col = Some(col);
    }

    pub fn to_string(&self) -> String {
        String::from(&self.msg)
    }
}

impl From<FromDecStrErr> for Error {
    fn from(e: FromDecStrErr) -> Error {
        Error::new(&String::from("failed to parse decimal value: ").add(&e.to_string()))
    }
}

impl From<FromStrRadixErr> for Error {
    fn from(e: FromStrRadixErr) -> Error {
        Error::new(&String::from("failed to parse hex value: ").add(&e.to_string()))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::new(&e.to_string())
    }
}

impl From<EvalexprError> for Error {
    fn from(e: EvalexprError) -> Error {
        Error::new(&e.to_string())
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_ref())
    }
}
