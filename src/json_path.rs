use std::fmt::{Display, Formatter};
use std::str::FromStr;

use qjsonrs::JsonToken;
use serde_json::Value;

use crate::json_path::PathElement::ValueLeaf;

#[derive(Debug)]
pub enum PathElement {
    Key(String),
    EmptyArray,
    ArrayIndex(usize),
    Object,
    ValueLeaf,
}

impl PathElement {
    fn is_array(&self) -> bool {
        match self {
            PathElement::EmptyArray => { true }
            PathElement::ArrayIndex(_) => { true }
            _ => { false }
        }
    }

    fn is_object(&self) -> bool {
        match self {
            PathElement::Object => { true }
            _ => { false }
        }
    }

    fn is_leaf(&self) -> bool {
        match self {
            ValueLeaf => { true }
            _ => { false }
        }
    }
}


pub trait TokenInfo {
    fn is_value(&self) -> bool;
    fn as_value(&self) -> Option<Value>;
}

impl TokenInfo for JsonToken<'_> {
    fn is_value(&self) -> bool {
        match self {
            JsonToken::JsNull => { true }
            JsonToken::JsBoolean(_) => { true }
            JsonToken::JsNumber(_) => { true }
            JsonToken::JsString(_) => { true }
            _ => { false }
        }
    }
    fn as_value(&self) -> Option<Value> {
        match self {
            JsonToken::JsNull => { Some(Value::Null) }
            JsonToken::JsBoolean(b) => { Some(Value::Bool(*b)) }
            JsonToken::JsNumber(n) => { Some(Value::Number(serde_json::Number::from_str(n).ok()?)) }
            JsonToken::JsString(s) => { Some(Value::String(s.clone().into_raw_str().to_string())) }
            _ => { None }
        }
    }
}

impl Display for PathElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            PathElement::Key(key) => { format!(".{key}") }
            PathElement::ArrayIndex(i) => { format!("[{i}]") }
            _ => { format!("") }
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug)]
pub struct JsonPath {
    elements: Vec<PathElement>,
}

impl JsonPath {
    pub fn new() -> JsonPath {
        JsonPath {
            elements: vec![],
        }
    }

    fn array_index_inc(&mut self) {
        if let Some(PathElement::EmptyArray) = self.elements.last() {
            self.replace_head(PathElement::ArrayIndex(0));
        } else if let Some(PathElement::ArrayIndex(i)) = self.elements.last() {
            self.replace_head(PathElement::ArrayIndex(i + 1));
        }
    }

    fn replace_head(&mut self, element: PathElement) {
        if let Some(last_index) = self.elements.len().checked_sub(1) {
            self.elements[last_index] = element;
        }
    }

    fn backtrack(&mut self) {
        loop {
            match self.elements.pop() {
                None => { break; }
                Some(element) => {
                    if element.is_object() || element.is_array() {
                        self.elements.push(element);
                        break;
                    }
                }
            }
        }
    }

    fn end_array(&mut self) {
        loop {
            match self.elements.pop() {
                None => { break; }
                Some(element) => {
                    if element.is_array() {
                        break;
                    }
                }
            }
        }
    }


    fn end_object(&mut self) {
        loop {
            match self.elements.pop() {
                None => { break; }
                Some(element) => {
                    if element.is_object() {
                        break;
                    }
                }
            }
        }
    }


    fn head_is_leaf(&self) -> bool {
        if let Some(PathElement::ValueLeaf) = self.elements.last() {
            true
        } else {
            false
        }
    }

    fn head_is_array(&self) -> bool {
        let head = self.elements.last();
        if let Some(PathElement::EmptyArray) = head {
            true
        } else if let Some(PathElement::ArrayIndex(_)) = head {
            true
        } else {
            false
        }
    }

    fn push_value(&mut self, element: PathElement) {
        if self.head_is_array() {
            self.array_index_inc();
        }


        self.elements.push(element);
    }

    fn push_key(&mut self, key: &str) {
        self.elements.push(PathElement::Key(key.to_string()));
    }

    pub fn push(&mut self, token: &JsonToken) {
        if self.head_is_leaf() {
            self.backtrack();
        }
        match token {
            JsonToken::StartObject => {
                self.push_value(PathElement::Object);
            }
            JsonToken::EndObject => {
                self.elements.pop();
                self.backtrack();
            }
            JsonToken::StartArray => {
                self.push_value(PathElement::EmptyArray);
            }
            JsonToken::EndArray => {
                self.end_array();
                self.backtrack();
            }
            JsonToken::JsNull => {
                self.push_value(ValueLeaf);
            }
            JsonToken::JsBoolean(_) => {
                self.push_value(ValueLeaf);
            }
            JsonToken::JsNumber(_) => {
                self.push_value(ValueLeaf);
            }
            JsonToken::JsString(_) => {
                self.push_value(ValueLeaf);
            }
            JsonToken::JsKey(s) => {
                self.push_key(s.clone().into_raw_str());
            }
        }
    }
}

impl Display for JsonPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let result = self.elements
            .iter()
            .map(|e| format!("{e}"))
            .collect::<Vec<String>>()
            .join("");

        if result.is_empty() || !result.starts_with(".") {
            write!(f, ".{result}")
        } else {
            write!(f, "{result}")
        }
    }
}


#[cfg(test)]
mod tests {
    use qjsonrs::JsonString;
    use qjsonrs::JsonToken::{EndArray, EndObject, JsKey, JsNull, JsString, StartArray, StartObject};
    use qjsonrs::sync::{Stream, TokenIterator};

    use crate::json_path::{JsonPath, PathElement, TokenInfo};

    #[test]
    fn test_format() {
        for i in 0..10 {
            assert_eq!(format!("{}", PathElement::ArrayIndex(i)), format!("[{i}]"))
        }
    }

    fn json_string(s: &str) -> JsonString {
        JsonString::from_str_ref(s).unwrap()
    }

    #[test]
    fn test_json_path() {
        let mut json_path = JsonPath::new();


        let steps = vec![
            (StartObject, "."),
            (JsKey(json_string("foo")), ".foo"),
            (StartArray, ".foo"),
            (JsString(json_string("bar")), ".foo[0]"),
            (JsString(json_string("car")), ".foo[1]"),
            (EndArray, "."),
            (JsKey(json_string("doo")), ".doo"),
            (StartObject, ".doo"),
            (JsKey(json_string("eol")), ".doo.eol"),
            (JsNull, ".doo.eol"),
            (EndObject, "."),
            (EndObject, "."),
        ];


        for (i, step) in steps.iter().enumerate() {
            let (token, expected) = step;

            let previous = format!("{json_path}");
            json_path.push(token);
            let actual = format!("{}", json_path);
            assert_eq!(actual, **expected, "Last token {:?}, Path: {:?}", token, json_path);
            println!("{previous} + {token:?} -> {json_path} ?= {expected}");
        }
    }


    #[test]
    fn test_token_stream() {
        let data = include_bytes!("../testdata/mars_weather.json");

        let mut stream = Stream::from_read(&data[..]).unwrap();
        let mut json_path = JsonPath::new();

        while let Ok(Some(token)) = stream.next() {
            json_path.push(&token);

            // println!("Token: {token:?}, Path: {json_path:?}");
            if let Some(e) = json_path.elements.first() {
                assert!(e.is_array());
            }
            if let Some(value) = token.as_value() {
                println!("{json_path}: {value}");
            }
        }
    }
}

