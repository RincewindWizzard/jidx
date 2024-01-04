use qjsonrs::JsonToken;

use crate::json_path::Value;
use crate::json_path::Value::{Boolean, Null, Number};

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
    ArrayIndex(usize)
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
            StackElement::JsNull => { Some(Null) }
            StackElement::JsBoolean(b) => { Some(Boolean(*b)) }
            StackElement::JsNumber(n) => { Some(Number(n.to_string())) }
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