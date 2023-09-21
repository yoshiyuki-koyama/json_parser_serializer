//! JSON Serializer module.
use super::{JsonKey, JsonValue, NumberType, JsonObject, JsonSerializerNewLineKind, JsonSerializerIndentKind};

use super::error::*;

fn serialize_error(kind: JsonErrorKind, detail_str: &str, status_str: &str) -> Box<dyn std::error::Error + Send + Sync + 'static> {
    return JsonError::new(kind, Some(format!("{} | {}",detail_str,status_str)));
}

#[derive(PartialEq)]
enum StartObjectKind {
    EmptyObject,
    HasSomeMember
}

const NEWLINE_STR_CRLF :&str = "\u{000D}\u{000A}";
const NEWLINE_STR_LF :&str = "\u{000A}";

/// JSON serializer struct.
#[derive(Clone, Debug)]
pub struct JsonSerializer {
    newline_str: &'static str,
    indent_string: String,
    indent_level: usize,
}


impl JsonSerializer {
    /// Serialize JSON function.
    #[allow(dead_code)]
    pub fn serialize(json_object: &JsonObject, newline_kind: JsonSerializerNewLineKind, indent_kind: JsonSerializerIndentKind) -> Result<String> {
        let mut json_serializer:JsonSerializer = JsonSerializer::new(newline_kind, indent_kind);

        let mut content_string = String::new();
        json_serializer.object_serializer(json_object, &mut content_string)?;
        // 最後に改行する
        content_string.push_str(&json_serializer.newline_str);
        Ok(content_string)
    }

    fn new(newline_kind: JsonSerializerNewLineKind, indent_kind: JsonSerializerIndentKind) -> JsonSerializer {
        let newline_str: &'static str = {
            match newline_kind {
                JsonSerializerNewLineKind::Crlf => {
                    NEWLINE_STR_CRLF
                }
                JsonSerializerNewLineKind::Lf => {
                    NEWLINE_STR_LF
                }
            }
        };
        let indent_string = {
            match indent_kind {
                JsonSerializerIndentKind::Tab => {
                    "\t".to_string()
                }
                JsonSerializerIndentKind::Space(length) => {
                    let mut tmp_string = String::new();
                    for _ in 0..length {
                        tmp_string.push(' ');
                    }
                    tmp_string
                }
            }
        };
        JsonSerializer {
            newline_str: newline_str,
            indent_string: indent_string,
            indent_level: 0,
        }
    }

    fn make_indent_string(&self) -> String{
        let mut indent_string = String::new();
        for _ in 0..self.indent_level {
            indent_string.push_str(&self.indent_string);
        }
        indent_string
    }

    fn object_serializer(&mut self, json_object: &JsonObject, content_string: &mut String) -> Result<()> {
        match self.start_object_serializer(json_object, content_string)? {
            StartObjectKind::EmptyObject => {
               return Ok(());
            }
            StartObjectKind::HasSomeMember => {
                let mut member_count: usize = 0;
                for (json_key, json_value) in &json_object.members {
                    self.key_serializer(json_key,content_string)?;
                    self.coron_serializer(content_string)?;
                    self.value_serializer(json_value, content_string)?;
                    if member_count < json_object.members.len() - 1 {
                        self.end_member_serializer(content_string)?
                    }
                    member_count += 1;
                }
                self.end_object_serializer(content_string)?
            }
        }
        Ok(())
    }

    fn start_object_serializer(&mut self, json_object: &JsonObject, content_string: &mut String) -> Result<StartObjectKind> {
        if json_object.members.len() == 0 {
            content_string.push_str("{}");
            return Ok(StartObjectKind::EmptyObject);
        }
        else {
            content_string.push_str("{");
            content_string.push_str(&self.newline_str);
            self.indent_level += 1;
            return Ok(StartObjectKind::HasSomeMember);
        }
    }

    fn key_serializer(&self, json_key: &JsonKey, content_string: &mut String) -> Result<()> {
        content_string.push_str(&self.make_indent_string());
        self.string_serializer(&json_key.0, content_string)?;
        Ok(())
    }

    fn coron_serializer(&self, content_string: &mut String) -> Result<()> {
        content_string.push_str(" : ");
        Ok(())
    }

    fn value_serializer(&mut self, json_value: &JsonValue, content_string: &mut String) -> Result<()> {
        match json_value {
            JsonValue::ValueString(json_string) => {
                self.string_serializer(json_string, content_string)?;
            }
            JsonValue::ValueNumber(json_number) => {
                self.number_serializer(json_number, content_string)?;
            }
            JsonValue::ValueBool(json_bool) => {
                self.bool_serializer(json_bool, content_string)?;
            }
            JsonValue::ValueNull => {
                self.null_serializer(content_string)?;
            }
            JsonValue::ValueArray(json_array) => {
                self.array_serializer(json_array, content_string)?
            }
            JsonValue::ValueObject(refcell_json_object) => {
                let json_object = refcell_json_object.borrow();
                self.object_serializer(&json_object, content_string)?
            }
        }
        Ok(())
    }

    fn end_member_serializer(&self, content_string: &mut String) -> Result<()> {
        content_string.push_str(",");
        content_string.push_str(&self.newline_str);
        Ok(())
    }

    fn end_object_serializer(&mut self, content_string: &mut String) -> Result<()> {
        self.indent_level -= 1;
        content_string.push_str(&self.newline_str);
        content_string.push_str(&self.make_indent_string());
        content_string.push_str("}");
        Ok(())
    }

    fn string_serializer(&self, json_string_str: &str, content_string: &mut String) -> Result<()> {
        content_string.push_str("\"");
        for unicode_char in json_string_str.chars() {
            match unicode_char {
                '\"' => {
                    content_string.push_str("\\\"");
                }
                '\\' => {
                    content_string.push_str("\\\\");
                }
                '\r' => {
                    content_string.push_str("\\r");
                }
                '\n' => {
                    content_string.push_str("\\n");
                }
                '\t' => {
                    content_string.push_str("\\t");
                }
                '\u{0008}' => {
                    content_string.push_str("\\b");
                }
                '\u{000C}' => {
                    content_string.push_str("\\f");
                }
                ('\u{0000}'..='\u{0007}') | '\u{000B}' | ('\u{000E}'..='\u{0001F}') => {
                    let u32_code_point = unicode_char as u32;
                    content_string.push_str(&format!("\\u{:04x}", u32_code_point));
                }
                _ => {
                    content_string.push(unicode_char);
                }
            }
        }
        content_string.push_str("\"");
        Ok(())
    }

    fn number_serializer(&self, json_number: &NumberType, content_string: &mut String) -> Result<()> {

        match json_number {
            NumberType::Int(int_number) => {
                content_string.push_str(&format!("{}", int_number));
            }
            NumberType::Float(float_number) => {
                if float_number.is_nan() || float_number.is_infinite() {
                    return Err(serialize_error(JsonErrorKind::SerializeErrorInNumber, "Number:  Number is NaN or Infinite.", &format!("{}", float_number)));
                }
                content_string.push_str(&format!("{}", float_number));
            }
        }
        Ok(())
    }

    fn bool_serializer(&self, json_bool: &bool, content_string: &mut String) -> Result<()> {
        if *json_bool {
            content_string.push_str("true");
        }
        else {
            content_string.push_str("false");
        }
        Ok(())
    }

    fn null_serializer(&self, content_string: &mut String) -> Result<()> {
        content_string.push_str("null");
        Ok(())
    }

    fn array_serializer(&mut self, json_array: &Vec<JsonValue>,  content_string: &mut String) -> Result<()> {
        content_string.push_str("[");

        for (idx, json_value) in json_array.iter().enumerate() {
            content_string.push_str(" ");
            match json_value {
                JsonValue::ValueString(json_string) => {
                    self.string_serializer(json_string, content_string)?;
                }
                JsonValue::ValueNumber(json_number) => {
                    self.number_serializer(json_number, content_string)?;
                }
                JsonValue::ValueBool(json_bool) => {
                    self.bool_serializer(json_bool, content_string)?;
                }
                JsonValue::ValueNull => {
                    self.null_serializer(content_string)?;
                }
                JsonValue::ValueArray(json_array) => {
                    self.array_serializer(json_array, content_string)?;
                }
                JsonValue::ValueObject(refcell_json_object) => {
                    content_string.push_str(self.newline_str);
                    if idx == 0 {
                        self.indent_level += 1;
                    }
                    content_string.push_str(&self.make_indent_string());
                    let json_object = refcell_json_object.borrow();
                    self.object_serializer(&json_object, content_string)?;
                    if idx == json_array.len() - 1 {
                        self.indent_level -= 1;
                        content_string.push_str(self.newline_str);
                        content_string.push_str(&self.make_indent_string());
                    }
                }
            }
            if idx < json_array.len() - 1 {
                content_string.push_str(",");
            }
        }
        content_string.push_str("]");
        Ok(())
    }
}
