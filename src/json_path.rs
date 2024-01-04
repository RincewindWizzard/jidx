use std::fmt::{Display, Formatter};

use qjsonrs::JsonString;

use crate::json_path::JsonPathElement::Key;
use crate::json_path::Value::{Boolean, Null, Number};
use crate::StackElement;

#[derive(Clone)]
pub struct JsonPath {
    elements: Vec<JsonPathElement>,
}

impl JsonPath {
    pub fn from(stack: &Vec<StackElement>) -> String {
        "".to_string()
    }
}

impl Default for JsonPath {
    fn default() -> Self {
        JsonPath::new()
    }
}

#[derive(Clone)]
pub enum Value {
    String(String),
    Number(String),
    Boolean(bool),
    Null,
}



impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(x) => { write!(f, "\"{x}\"") }
            Value::Number(x) => { write!(f, "{x}") }
            Value::Boolean(x) => { write!(f, "{x}") }
            Value::Null => { write!(f, "null") }
            _ => { Ok(()) }
        }
    }
}

#[derive(Clone)]
pub enum JsonPathElement {
    Key(String),
    Index(usize),
}

impl JsonPath {
    pub fn new() -> JsonPath {
        JsonPath {
            elements: vec![],
        }
    }
    pub fn push<T>(&mut self, x: T)
        where
            T: Into<JsonPathElement>,
    {
        self.elements.push(x.into());
    }

    pub fn pop(&mut self) -> Option<JsonPathElement> {
        self.elements.pop()
    }
}

impl Display for JsonPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.elements.is_empty() {
            write!(f, ".")
        } else {
            let s = self.elements
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join("");
            write!(f, "{}", s)
        }
    }
}

impl Display for JsonPathElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Key(key) => {
                write!(f, ".{}", key)
            }
            JsonPathElement::Index(index) => {
                write!(f, "[{}]", index)
            }
        }
    }
}

impl From<JsonString<'_>> for JsonPathElement {
    fn from(value: JsonString) -> Self {
        let key: String = value.into_raw_str().to_string();
        Key(key)
    }
}