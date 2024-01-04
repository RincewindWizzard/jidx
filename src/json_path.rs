use std::fmt::{Display, Formatter};

use crate::token::StackElement;

pub(crate) type JsonPath = String;

pub trait ToJsonPath {
    fn as_json_path(&self) -> JsonPath;
}


impl ToJsonPath for StackElement {
    fn as_json_path(&self) -> JsonPath {
        match self {
            StackElement::JsNull => { "null".to_string() }
            StackElement::JsBoolean(b) => { format!("{b}") }
            StackElement::JsNumber(n) => { format!("{n}") }
            StackElement::JsString(s) => { format!("\"{s}\"") }
            StackElement::JsKey(key) => { format!("{key}") }
            StackElement::ArrayIndex(i) => { format!("[{i}]") }

            StackElement::StartObject => { format!(".") }
            _ => { "".to_string() }
        }
    }
}

impl ToJsonPath for Vec<StackElement> {
    fn as_json_path(&self) -> JsonPath {
        let xs: Vec<&StackElement> = self
            .iter()
            .filter(|e| match e {
                StackElement::StartObject => { true }
                StackElement::EndObject => { true }
                StackElement::StartArray => { true }
                StackElement::EndArray => { true }
                StackElement::JsKey(_) => { true }
                StackElement::ArrayIndex(_) => { true }
                _ => { false }
            }).collect();

        // this is a fatal error and a programming error
        // we cannot regenerate from this error
        let mut last = false;
        for cur in &xs {
            if let StackElement::JsKey(_) = cur {
                if last {
                    panic!("Two JsKeys without separator!");
                }
                last = true;
            } else {
                last = false;
            }
        }

        let xs: Vec<JsonPath> = xs.iter()
            .map(|e| e.as_json_path())
            .collect();
        xs.join("")
    }
}


#[derive(Clone)]
pub enum Value {
    String(String),
    Number(String),
    Boolean(bool),
    EmptyArray,
    Null,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(x) => { write!(f, "\"{x}\"") }
            Value::Number(x) => { write!(f, "{x}") }
            Value::Boolean(x) => { write!(f, "{x}") }
            Value::Null => { write!(f, "null") }
            Value::EmptyArray => { write!(f, "[]") }
            _ => { Ok(()) }
        }
    }
}


//
// #[derive(Clone)]
// pub struct JsonPath {
//     elements: Vec<JsonPathElement>,
// }
//
// impl JsonPath {
//     pub fn from(stack: &Vec<StackElement>) -> String {
//         "".to_string()
//     }
// }
//
// impl Default for JsonPath {
//     fn default() -> Self {
//         JsonPath::new()
//     }
// }
//
// #[derive(Clone)]
// pub enum Value {
//     String(String),
//     Number(String),
//     Boolean(bool),
//     Null,
// }
//
//
//
// impl Display for Value {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Value::String(x) => { write!(f, "\"{x}\"") }
//             Value::Number(x) => { write!(f, "{x}") }
//             Value::Boolean(x) => { write!(f, "{x}") }
//             Value::Null => { write!(f, "null") }
//             _ => { Ok(()) }
//         }
//     }
// }
//
// #[derive(Clone)]
// pub enum JsonPathElement {
//     Key(String),
//     Index(usize),
// }
//
// impl JsonPath {
//     pub fn new() -> JsonPath {
//         JsonPath {
//             elements: vec![],
//         }
//     }
//     pub fn push<T>(&mut self, x: T)
//         where
//             T: Into<JsonPathElement>,
//     {
//         self.elements.push(x.into());
//     }
//
//     pub fn pop(&mut self) -> Option<JsonPathElement> {
//         self.elements.pop()
//     }
// }
//
// impl Display for JsonPath {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         if self.elements.is_empty() {
//             write!(f, ".")
//         } else {
//             let s = self.elements
//                 .iter()
//                 .map(|x| format!("{}", x))
//                 .collect::<Vec<String>>()
//                 .join("");
//             write!(f, "{}", s)
//         }
//     }
// }
//
// impl Display for JsonPathElement {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Key(key) => {
//                 write!(f, ".{}", key)
//             }
//             JsonPathElement::Index(index) => {
//                 write!(f, "[{}]", index)
//             }
//         }
//     }
// }
//
// impl From<JsonString<'_>> for JsonPathElement {
//     fn from(value: JsonString) -> Self {
//         let key: String = value.into_raw_str().to_string();
//         Key(key)
//     }
// }