use std::fmt::{Display, Formatter};

use qjsonrs::JsonToken;

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


    pub fn push(&mut self, token: &JsonToken) {
        match token {
            JsonToken::StartObject => {}
            JsonToken::EndObject => {
                let head = self.elements.last();
                if let Some(PathElement::Key(_)) = head {
                    self.elements.pop();
                }
            }
            JsonToken::StartArray => {
                self.elements.push(PathElement::EmptyArray);
            }
            JsonToken::EndArray => {
                let head = self.elements.last();
                if let Some(PathElement::EmptyArray) = head {
                    self.elements.pop();
                } else if let Some(PathElement::ArrayIndex(_)) = head {
                    self.elements.pop();
                }

                if let Some(PathElement::Key(key)) = self.elements.last() {
                    self.elements.pop();
                } else {
                    panic!("Json Path is corrupted! {self:?}");
                }
            }
            JsonToken::JsNull => {
                self.array_index_inc();
            }
            JsonToken::JsBoolean(_) => {
                self.array_index_inc();
            }
            JsonToken::JsNumber(_) => {
                self.array_index_inc();
            }
            JsonToken::JsString(_) => {
                self.array_index_inc();
            }
            JsonToken::JsKey(s) => {
                self.elements.push(PathElement::Key(s.clone().into_raw_str().to_string()));
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
    use qjsonrs::JsonString;
    use qjsonrs::JsonToken::{EndArray, EndObject, JsKey, JsNull, JsString, StartArray, StartObject};
    use qjsonrs::sync::{Stream, TokenIterator};

    use crate::json_path::{JsonPath, PathElement};

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

    #[test]
    fn test_token_stream() {
        let data = include_bytes!("../testdata/mars_weather.json");

        let mut stream = Stream::from_read(&data[..]).unwrap();

        while let Ok(Some(token)) = stream.next() {
            println!("{token:?}");
        }
    }
}