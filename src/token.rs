use std::fs::File;
use std::str::FromStr;
use log::debug;
use qjsonrs::JsonToken;
use qjsonrs::sync::{Stream, TokenIterator};
use serde_json::Value;


use crate::json_path::{JsonPath, ToJsonPath};
use crate::token::StackElement::ArrayIndex;

/// A token from a stream of JSON.
#[derive(Debug, PartialEq, Clone)]
pub enum StackElement {
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
    /// Represents the current position in the array
    ArrayIndex(usize),
}

impl StackElement {
    pub fn is_pop_token(&self) -> bool {
        match self {
            StackElement::EndObject => { true }
            StackElement::EndArray => { true }
            _ => { false }
        }
    }
    pub fn as_value(&self) -> Option<Value> {
        match self {
            StackElement::JsNull => { Some(Value::Null) }
            StackElement::JsBoolean(b) => { Some(Value::Bool(*b)) }
            StackElement::JsNumber(n) => { Some(Value::Number(serde_json::Number::from_str(n).ok()?)) }
            StackElement::JsString(s) => { Some(Value::String(s.clone())) }
            _ => { None }
        }
    }
}

impl From<JsonToken<'_>> for StackElement {
    fn from(token: JsonToken) -> Self {
        match token {
            JsonToken::StartObject => { StackElement::StartObject }
            JsonToken::EndObject => { StackElement::EndObject }
            JsonToken::StartArray => { StackElement::StartArray }
            JsonToken::EndArray => { StackElement::EndArray }
            JsonToken::JsNull => { StackElement::JsNull }
            JsonToken::JsBoolean(b) => { StackElement::JsBoolean(b) }
            JsonToken::JsNumber(n) => { StackElement::JsNumber(n.to_string()) }
            JsonToken::JsString(s) => { StackElement::JsString(s.into_raw_str().to_string()) }
            JsonToken::JsKey(key) => { StackElement::JsKey(key.into_raw_str().to_string()) }
        }
    }
}

pub struct JsonIndexIterator {
    token_stack: Vec<StackElement>,
    stream: Stream<File>,
}

pub trait JsonIndexable {
    fn index_paths(self) -> JsonIndexIterator;
}

impl JsonIndexable for Stream<File> {
    fn index_paths(self) -> JsonIndexIterator {
        JsonIndexIterator {
            token_stack: vec![],
            stream: self,
        }
    }
}


impl JsonIndexIterator {
    fn clean_stack(&mut self) {
        if let Some(StackElement::EndObject) = self.token_stack.last() {
            loop {
                let popped = self.token_stack.pop();
                debug!("popped: {:?}", popped);
                debug!("Stack: {:?}", self.token_stack);
                match popped {
                    None => { break; }
                    Some(x) => {
                        match x {
                            StackElement::StartObject => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        loop {
            let popped = self.token_stack.pop();
            debug!("popped: {:?}", popped);
            debug!("Stack: {:?}", self.token_stack);
            match popped {
                None => { break; }
                Some(x) => {
                    match x {
                        StackElement::JsKey(_) => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn inc_array_index(&mut self) -> usize {
        if let Some(StackElement::ArrayIndex(n)) = self.token_stack.last() {
            let last_index = self.token_stack.len() - 1;
            let n = n + 1;
            self.token_stack[last_index] = StackElement::ArrayIndex(n);
            return n;
        }
        0
    }
    pub(crate) fn next(&mut self) -> anyhow::Result<Option<(JsonPath, Value)>> {
        loop {
            match self.stream.next()? {
                None => {
                    return Ok(None);
                }
                Some(token) => {
                    let token = StackElement::from(token);
                    debug!("Got token: {:?}", token);
                    self.token_stack.push(token.clone());
                    debug!("Stack: {:?}", self.token_stack);


                    // push array index onto the stack
                    if let StackElement::StartArray = token {
                        self.token_stack.push(ArrayIndex(0));
                    }

                    // return empty array as single value
                    if let StackElement::EndArray = token {
                        let last_index = self.token_stack.len();
                        let array_len = match &self.token_stack[last_index - 2] {
                            ArrayIndex(i) => { *i }
                            _ => { 0 }
                        };
                        debug!("Array len: {array_len}");
                        let path = self.token_stack.as_json_path();
                        self.clean_stack();
                        if array_len == 0 {
                            return Ok(Some((path, Value::Array(vec![]))));
                        }
                    }


                    if let Some(value) = token.as_value() {
                        debug!("Found value: {value}");
                        let path = self.token_stack.as_json_path();
                        // remove token from stack
                        self.token_stack.pop();
                        let index = self.inc_array_index();
                        if index == 0 {
                            self.clean_stack();
                        }
                        return Ok(Some((path, value)));
                    }


                    if let StackElement::EndObject = token {
                        self.clean_stack();
                    }
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::File;
    use qjsonrs::sync::Stream;
    use crate::token::JsonIndexable;

    const EXAMPLE_JSON: &str = "./testdata/mars_weather.json";

    #[test]
    fn test_parse_doc() {
        let file = File::open(crate::EXAMPLE_JSON).unwrap();
        let mut stream = Stream::from_read(file).unwrap().index_paths();

        let mut result = HashMap::new();
        loop {
            match stream.next().unwrap() {
                None => {
                    break;
                }
                Some((key, value)) => {
                    result.insert(key, value);
                }
            }
        }

        assert_eq!(result.len(), 35);
        let parsed: serde_json::Value = serde_json::from_str(include_str!("../testdata/mars_weather_flattened.json"))
            .expect("Failed to parse JSON");

        if let serde_json::Value::Object(map) = parsed {
            for (key, value) in map {
                let actual = result.get(&key);
                assert!(actual.is_some(), "Missing key: {}", key);
                assert_eq!(*actual.unwrap(), value);
            }
        } else {
            panic!("Could not parse expected result!");
        }
    }
}