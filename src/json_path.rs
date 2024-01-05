use std::fmt::{Display, Formatter};

use crate::json_path::JsToken::JsKey;
use crate::json_path::PathElement::Key;

/// A token from a stream of JSON.
#[derive(Debug, PartialEq, Clone)]
pub enum JsToken {
    /// The start of an object, a.k.a. '{'
    StartObject,
    /// The end of an object, a.k.a. '}'
    EndObject,
    /// The start of an array, a.k.a. '['
    StartArray,
    /// The end of an object, a.k.a. ']'
    EndArray,
    /// The token 'null'
    JsNull,
    /// Either 'true' or 'false'
    JsBoolean(bool),
    /// A number, unparsed. i.e. '-123.456e-789'
    JsNumber(String),
    /// A JSON string in a value context.
    JsString(String),
    /// A JSON string in the context of a key in a JSON object.
    JsKey(String),
}


impl JsToken {
    fn is_value(&self) -> bool {
        match self {
            JsToken::JsNull => { true }
            JsToken::JsBoolean(_) => { true }
            JsToken::JsNumber(_) => { true }
            JsToken::JsString(_) => { true }
            _ => { false }
        }
    }
}

#[derive(Debug)]
pub enum PathElement {
    Key(String),
    EmptyArray,
    ArrayIndex(usize),
}

impl Display for PathElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            PathElement::Key(key) => { format!(".{key}") }
            PathElement::ArrayIndex(i) => { format!("[{i}]") }
            PathElement::EmptyArray => { format!("") }
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

    fn remove_branch(&mut self) {
        loop {
            match self.elements.pop() {
                None => { break; }
                Some(element) => {
                    match element {
                        PathElement::Key(_) => { break; }
                        PathElement::EmptyArray => {}
                        PathElement::ArrayIndex(_) => {}
                    }
                }
            }
        }
    }


    pub fn push(&mut self, token: &JsToken) {
        match token {
            JsToken::StartObject => {}
            JsToken::EndObject => {
                let head = self.elements.last();
                if let Some(PathElement::Key(_)) = head {
                    self.elements.pop();
                }
            }
            JsToken::StartArray => {
                self.elements.push(PathElement::EmptyArray);
            }
            JsToken::EndArray => {
                let head = self.elements.last();
                if let Some(PathElement::EmptyArray) = head {
                    self.elements.pop();
                } else if let Some(PathElement::ArrayIndex(_)) = head {
                    self.elements.pop();
                }

                if let Some(Key(key)) = self.elements.last() {
                    self.elements.pop();
                } else {
                    panic!("Json Path is corrupted! {self:?}");
                }
            }
            JsToken::JsNull => {
                self.array_index_inc();
            }
            JsToken::JsBoolean(_) => {
                self.array_index_inc();
            }
            JsToken::JsNumber(_) => {
                self.array_index_inc();
            }
            JsToken::JsString(_) => {
                self.array_index_inc();
            }
            JsToken::JsKey(s) => {
                self.elements.push(PathElement::Key(s.clone()));
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

        if result.is_empty() {
            write!(f, ".")
        } else {
            write!(f, "{result}")
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::json_path::{JsonPath, PathElement};
    use crate::json_path::JsToken::{EndArray, EndObject, JsKey, JsNull, JsString, StartArray, StartObject};

    #[test]
    fn test_format() {
        for i in 0..10 {
            assert_eq!(format!("{}", PathElement::ArrayIndex(i)), format!("[{i}]"))
        }
    }



    #[test]
    fn test_json_path() {
        let mut json_path = JsonPath::new();

        let steps = vec![
            (StartObject, "."),
            (JsKey("foo".to_string()), ".foo"),
            (StartArray, ".foo"),
            (JsString("bar".to_string()), ".foo[0]"),
            (JsString("car".to_string()), ".foo[1]"),
            (EndArray, "."),
            (JsKey("doo".to_string()), ".doo"),
            (StartObject, ".doo"),
            (JsKey("eol".to_string()), ".doo.eol"),
            (JsNull, ".doo.eol"),
            (EndObject, ".doo"),
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
}