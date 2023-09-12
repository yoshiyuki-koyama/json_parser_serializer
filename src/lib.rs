//! This libary provides the following features:
//! * Reading & writing json files.
use std::rc::Rc;
use std::collections::HashMap;

mod error;

mod parser;

mod tests;



#[derive(Clone, Copy, PartialEq, Debug)]
pub enum JsonNumber {
    JsonInt(i64),
    JsonFloat(f64),
}


/// JSON structure.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct JsonKey(String);

impl JsonKey {
    pub fn new() -> JsonKey {
        JsonKey(String::new())
    }
}

#[derive(Clone, Debug)]
pub enum JsonValue {
    ValueObject(Rc<JsonObject>),
    ValueArray(Vec<JsonValue>),
    ValueString(String),
    ValueNumber(JsonNumber),
    ValueBool(bool),
    ValueNull,
    None,
}

#[derive(Clone, Debug)]
pub struct JsonObject {
    pub members: HashMap<JsonKey, JsonValue>,
}

impl JsonObject {
    pub fn new(
    ) -> JsonObject {
        JsonObject {
            members: HashMap::new(),
        }
    }
}
