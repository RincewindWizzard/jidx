use std::fmt::{Debug, Display};
use std::fs::File;

use clap::Parser;
use log::debug;
use qjsonrs::sync::{Stream, TokenIterator};

use crate::args::Args;
use crate::json_path::{JsonPath, Value};
use crate::json_path::ToJsonPath;
use crate::json_path::Value::EmptyArray;
use crate::token::StackElement;
use crate::token::StackElement::ArrayIndex;

mod json_path;
mod token;
mod args;

const EXAMPLE_JSON: &str = "./mars_weather.json";
const BUFFER_SIZE: usize = 1024;


fn main() {
    let args = Args::parse();
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init().expect("Could not setup logging!");
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
    fn next(&mut self) -> anyhow::Result<Option<(JsonPath, Value)>> {
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
                    let path = self.token_stack.as_json_path();


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
                        self.clean_stack();
                        if array_len == 0 {
                            return Ok(Some((path, EmptyArray)));
                        }
                    }


                    if let Some(value) = token.as_value() {
                        debug!("Found value: {value}");
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

fn read_json() -> anyhow::Result<()> {
    let file = File::open(EXAMPLE_JSON)?;
    let mut stream = Stream::from_read(file)?.index_paths();

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
