use std::fmt::{Display, Formatter};
use std::str::FromStr;

use qjsonrs::JsonToken;
use qjsonrs::sync::{Error, Stream, TokenIterator};
use serde_json::Value;
use serde_json::Value::Array;

use crate::json_path::PathElement::ValueLeaf;

#[derive(Debug)]
#[derive(Clone)]
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

#[derive(Debug, Clone)]
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

    fn head_is_empty_array(&self) -> bool {
        let head = self.elements.last();
        if let Some(PathElement::EmptyArray) = head {
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
                self.elements.push(ValueLeaf);
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

pub struct JsonIndexIterator<R>
{
    path: JsonPath,
    stream: qjsonrs::sync::Stream<R>,

}

impl<R> JsonIndexIterator<R>
    where
        R: std::io::Read
{
    fn from(stream: Stream<R>) -> JsonIndexIterator<R> {
        JsonIndexIterator {
            path: JsonPath::new(),
            stream,
        }
    }
    fn next(&mut self) -> Result<Option<(JsonPath, Value)>, Error> {
        loop {
            match self.stream.next()? {
                None => { return Ok(None); }
                Some(token) => {
                    let array_end = if let JsonToken::EndArray = token { true } else { false };
                    let empty_array = self.path.head_is_empty_array() && array_end;


                    self.path.push(&token);
                    // println!("Token: {token:?}, Path: {:?}", self.path);
                    if token.is_value() || empty_array {
                        // println!("{}: {:?}", self.path, token);
                        let path = self.path.clone();
                        return Ok(Some(
                            if empty_array {
                                (path, Array(vec![]))
                            } else {
                                (path, token.as_value().expect("We already checked that this is a value."))
                            }
                        ));
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Cursor;

    use qjsonrs::JsonString;
    use qjsonrs::JsonToken::{EndArray, EndObject, JsKey, JsNull, JsString, StartArray, StartObject};
    use qjsonrs::sync::{Stream, TokenIterator};

    use crate::json_path::{JsonIndexIterator, JsonPath, PathElement, TokenInfo};

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


        for (_, step) in steps.iter().enumerate() {
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

            println!("Token: {token:?}, Path: {json_path:?}");
            if let Some(e) = json_path.elements.first() {
                assert!(e.is_array() || e.is_leaf());
            }
            if let Some(value) = token.as_value() {
                println!("{json_path}: {value}");
            }
        }
    }

    #[test]
    fn test_array_in_arrays() {
        let data = "[[1, 2],[3],[], null]";
        let expected = [
            ".[0][0] -> 1",
            ".[0][1] -> 2",
            ".[1][0] -> 3",
            ".[2] -> []",
            ".[3] -> null"
        ];

        let mut stream = JsonIndexIterator::from(Stream::from_read(Cursor::new(data)).unwrap());

        let mut result = vec![];
        loop {
            match stream.next().unwrap() {
                None => {
                    break;
                }
                Some((key, value)) => {
                    let s = format!("{key} -> {value}");
                    println!("{}", s);
                    result.push(s);
                }
            }
        }

        for i in 0..expected.len() {
            assert_eq!(result[i], expected[i]);
        }
    }

    #[test]
    fn test_iterator() {
        let file = File::open("./testdata/mars_weather.json").unwrap();
        let mut stream = JsonIndexIterator::from(Stream::from_read(file).unwrap());
        loop {
            match stream.next().unwrap() {
                None => {
                    break;
                }
                Some((key, value)) => {
                    println!("{key} -> {value}");
                }
            }
        }
    }
}

