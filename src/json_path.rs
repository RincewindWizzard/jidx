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




/*impl Display for Value {
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
}*/
