#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

use std::collections::HashMap;
use std::fmt::Display;
use std::str;
use std::vec::Vec;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    Symbol(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", num),
            Self::Symbol(sym) => write!(f, "'{}", sym),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Interpreter {
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::<Value>::new(),
            variables: HashMap::<String, Value>::new(),
        }
    }

    fn get_number(&mut self) -> f64 {
        match self.stack.pop().unwrap() {
            Value::Number(number) => number,
            Value::Symbol(_) => panic!("Got symbol, expected number"),
        }
    }

    fn get_symbol(&mut self) -> String {
        match self.stack.pop().unwrap() {
            Value::Number(_) => panic!("Got number, expected symbol"),
            Value::Symbol(symbol) => symbol,
        }
    }

    pub fn eval(&mut self, expr: &str) {
        for token in expr.split_whitespace() {
            match token.parse::<f64>() {
                Ok(number) => {
                    self.stack.push(Value::Number(number));
                    continue;
                }
                Err(_error) => {}
            }
            if token.len() == 1 {
                let first = self.get_number();
                let second = self.get_number();
                let op = token.chars().next().unwrap();
                let result = match op {
                    '+' => first + second,
                    '-' => first - second,
                    '/' => first / second,
                    '*' => first * second,
                    _ => panic!("Wrong token {}", op),
                };
                self.stack.push(Value::Number(result));
            } else if token == "set" {
                let symbol: String = self.get_symbol();
                let value = self.stack.pop().unwrap();
                self.variables.insert(symbol, value);
            } else if token.starts_with('\'') {
                let symbol = &token[1..token.len()];
                self.stack.push(Value::Symbol(symbol.to_string()));
            } else if token.starts_with('$') {
                let symbol = &token[1..token.len()];
                let value = self.variables.get(&symbol.to_string()).unwrap();
                self.stack.push(value.clone());
            } else {
                panic!("Invalid token");
            }
        }
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
