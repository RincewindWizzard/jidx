use std::fmt::{Debug, Display};
use std::fs::File;

use qjsonrs::sync::{Stream, TokenIterator};

use crate::json_path::{JsonPath, Value};
use crate::token::StackElement;
use crate::token::StackElement::ArrayIndex;

mod json_path;
mod token;

const EXAMPLE_JSON: &str = "./mars_weather.json";
const BUFFER_SIZE: usize = 1024;


fn main() {
    read_json().unwrap();
}


struct JsonIndexIterator {
    token_stack: Vec<StackElement>,
    stream: Stream<File>,
}

pub trait JsonIndexable<'a> {
    fn index_paths(self) -> JsonIndexIterator;
}

impl<'a> JsonIndexable<'a> for Stream<File> {
    fn index_paths(self) -> JsonIndexIterator {
        JsonIndexIterator {
            token_stack: vec![],
            stream: self,
        }
    }
}


impl JsonIndexIterator {
    fn next(&mut self) -> anyhow::Result<Option<(String, Value)>> {
        loop {
            match self.stream.next()? {
                None => {
                    return Ok(None);
                }
                Some(token) => {
                    let token = StackElement::from(token);
                    let value = token.as_value();

                    println!("{:?}", self.token_stack);
                    if let Some(value) = value {
                        // TODO: yield value
                        let path = JsonPath::from(&self.token_stack);

                        if let Some(StackElement::ArrayIndex(n)) = self.token_stack.last() {
                            let last_index = self.token_stack.len() - 1;
                            self.token_stack[last_index] = StackElement::ArrayIndex(n + 1);
                        }

                        return Ok(Some((path, value)));
                    } else {
                        if token.is_pop_token() {
                            let parent = self.token_stack.pop();
                            println!("popped: {:?}", parent);
                            if let Some(parent) = parent {
                                self.token_stack.pop();
                            }
                        } else {
                            let is_array_start = if let StackElement::StartArray = token { true } else { false };
                            self.token_stack.push(token);
                            if is_array_start {
                                self.token_stack.push(ArrayIndex(0));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }
}

fn read_json() -> anyhow::Result<()> {
    use crate::json_path::Value;
    let file = File::open(EXAMPLE_JSON)?;
    let mut stream = Stream::from_read(file)?.index_paths();

    let mut path: JsonPath = JsonPath::new();
    let mut value: Option<Value> = None;

    loop {
        match stream.next()? {
            None => {
                break;
            }
            Some((key, value)) => {
                println!("Token: {key} -> {value}");
            }
        }
    }
    Ok(())
}
