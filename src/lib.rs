//! JSON Parser & Serializer library.
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

mod error;
mod parser;
mod serializer;
use parser::JsonParser;
use serializer::JsonSerializer;
use error::*;



mod tests;

/// JSON Key struct. JsonKey(String)
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct JsonKey(String);

/// JSON Value's enum.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    ValueString(String),
    ValueNumber(NumberType),
    ValueBool(bool),
    ValueNull,
    ValueArray(Vec<JsonValue>),
    ValueObject(Rc<RefCell<JsonObject>>),
}

/// JSON Number Value's enum.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NumberType {
    Int(i64),
    Float(f64),
}

/// JSON Object struct.
#[derive(Clone, PartialEq, Debug)]
pub struct JsonObject {
    pub members: HashMap<JsonKey, JsonValue>,
}

impl JsonObject {
    /// Create new empty JSON Onject.
    /// * Return:
    ///     * JSON Object struct.
    pub fn new(
    ) -> JsonObject {
        JsonObject {
            members: HashMap::new(),
        }
    }

    /// Parse JSON string to JSON Onject.
    /// * Parameters:
    ///     * `content_str` : JSON string(&str).
    /// * Return:
    ///     * JSON Object struct.
    pub fn parse(content_str:  &str) -> Result<JsonObject> {
        JsonParser::parse(content_str)
    }

    /// Serialize JSON object to string.
    /// * Parameters:
    ///     * `json_object` : JSON Object struct. 
    ///     * `newline_kind` : Newline code(LF or CRLF) when serializing JSON.
    ///     * `indent_kind` : Indent kind(Tab of Space) when serializing JSON.
    /// * Return:
    ///     * JSON string.
    pub fn serialize(&self, newline_kind: JsonSerializerNewLineKind, indent_kind: JsonSerializerIndentKind) -> Result<String> {
        JsonSerializer::serialize(self, newline_kind, indent_kind)
    }
}

/// Enum that specifies newline code(LF or CRLF) when serializing JSON.
#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum JsonSerializerNewLineKind {
    Lf,
    CrLf
}

/// Enum that specifies indent kind(Tab of Space) when serializing JSON. `Space(4)` means that specifies 4 spaces as indent.
#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum JsonSerializerIndentKind {
    Tab,
    Space(usize),
}
