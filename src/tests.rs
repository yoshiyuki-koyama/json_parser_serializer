#[cfg(test)]
mod tests {

    use crate::{JsonObject, JsonKey, JsonValue, NumberType, JsonSerializerNewLineKind};
    use super::super::error::*;

    use std::rc::Rc;
    use std::cell::RefCell;
    use std::fs::{create_dir, File};
    use std::io::prelude::*;
    use std::path::Path;

    fn member_assert_eq(json_object: &JsonObject, json_key_str: &str, expect_value: &JsonValue) {
        let parsed_value = json_object.members.get(&JsonKey(json_key_str.to_string())).unwrap();
        assert_eq!(parsed_value, expect_value); 
    }

    #[test]
    fn parse_string() -> Result<()> {
        let test_path = Path::new("./for_test/parse_test_string.json");
        let mut file = File::open(test_path)?;
        let mut content_string = String::new();
        file.read_to_string(&mut content_string)?;

        let json_object = JsonObject::parse(&content_string).unwrap();
        dbg!(json_object.clone());
        member_assert_eq(&json_object, "string", &JsonValue::ValueString("string".to_string()));
        member_assert_eq(&json_object, "escape_string1", &JsonValue::ValueString("escape_string1 doublequate:\" reversesolidus:\\ solidus:/".to_string()));
        member_assert_eq(&json_object, "escape_string2", &JsonValue::ValueString("escape_string2 BS:\u{0008} FF:\u{000C} LF:\n CR:\r HT:\t".to_string()));
        member_assert_eq(&json_object, "escape_string3", &JsonValue::ValueString("escape_string3 A:A あ:あ GlowingStar:🌟".to_string()));
        assert_eq!(json_object.members.len(), 4);
        Ok(())
    }

    #[test]
    fn parse_number() -> Result<()> {
        let test_path = Path::new("./for_test/parse_test_number.json");
        let mut file = File::open(test_path)?;
        let mut content_string = String::new();
        file.read_to_string(&mut content_string)?;

        let json_object = JsonObject::parse(&content_string).unwrap();
        dbg!(json_object.clone());
        member_assert_eq(&json_object, "int", &JsonValue::ValueNumber(NumberType::Int(1)));
        member_assert_eq(&json_object, "minus_int", &JsonValue::ValueNumber(NumberType::Int(-1)));
        member_assert_eq(&json_object, "float", &JsonValue::ValueNumber(NumberType::Float(0.1)));
        member_assert_eq(&json_object, "minus_float", &JsonValue::ValueNumber(NumberType::Float(-0.1)));
        member_assert_eq(&json_object, "exp", &JsonValue::ValueNumber(NumberType::Float(1500.0)));
        member_assert_eq(&json_object, "minus_EXP", &JsonValue::ValueNumber(NumberType::Float(-1500.0)));
        member_assert_eq(&json_object, "exp_plus", &JsonValue::ValueNumber(NumberType::Float(1500.0)));
        member_assert_eq(&json_object, "minus_exp_minus", &JsonValue::ValueNumber(NumberType::Float(-0.0015)));
        assert_eq!(json_object.members.len(), 8);
        Ok(())
    }

    #[test]
    fn parse_bool_null() -> Result<()> {
        let test_path = Path::new("./for_test/parse_test_bool_null.json");
        let mut file = File::open(test_path)?;
        let mut content_string = String::new();
        file.read_to_string(&mut content_string)?;

        let json_object = JsonObject::parse(&content_string).unwrap();
        dbg!(json_object.clone());
        member_assert_eq(&json_object, "true", &JsonValue::ValueBool(true));
        member_assert_eq(&json_object, "false", &JsonValue::ValueBool(false));
        member_assert_eq(&json_object, "null", &JsonValue::ValueNull);
        assert_eq!(json_object.members.len(), 3);
        Ok(())
    }

    #[test]
    fn parse_array() -> Result<()> {
        let test_path = Path::new("./for_test/parse_test_array.json");
        let mut file = File::open(test_path)?;
        let mut content_string = String::new();
        file.read_to_string(&mut content_string)?;

        let json_object = JsonObject::parse(&content_string).unwrap();
        dbg!(json_object.clone());
        member_assert_eq(&json_object, "empty_array", &JsonValue::ValueArray(vec![]));
        member_assert_eq(&json_object, "array_string", &JsonValue::ValueArray(vec![
            JsonValue::ValueString("string1".to_string()), 
            JsonValue::ValueString("string2".to_string()), 
            JsonValue::ValueString("string3".to_string())
            ]));
        member_assert_eq(&json_object, "array_number", &JsonValue::ValueArray(vec![
            JsonValue::ValueNumber(NumberType::Int(1)), 
            JsonValue::ValueNumber(NumberType::Int(2)), 
            JsonValue::ValueNumber(NumberType::Int(3)), 
            JsonValue::ValueNumber(NumberType::Int(4)), 
            JsonValue::ValueNumber(NumberType::Int(5)), 
            ]));
        member_assert_eq(&json_object, "array_bool_null", &JsonValue::ValueArray(vec![
            JsonValue::ValueBool(true), 
            JsonValue::ValueBool(false), 
            JsonValue::ValueNull, 
            ]));
        member_assert_eq(&json_object, "array_array", &JsonValue::ValueArray(vec![
            JsonValue::ValueArray(vec![ JsonValue::ValueString("0,0".to_string()),  JsonValue::ValueString("0,1".to_string())]),
            JsonValue::ValueArray(vec![ JsonValue::ValueString("1,0".to_string()),  JsonValue::ValueString("1,1".to_string())]),
            ]));

        if let JsonValue::ValueArray(json_array) = json_object.members.get(&JsonKey("array_object".to_string())).unwrap() {
            if let JsonValue::ValueObject(child_json_object) = &json_array[0] {
                member_assert_eq(&child_json_object.borrow(), "array_object_string", &JsonValue::ValueString("array_objct_string1".to_string()));
                member_assert_eq(&child_json_object.borrow(), "array_object_number", &JsonValue::ValueNumber(NumberType::Float(1.0)));
                assert_eq!(child_json_object.borrow().members.len(), 2);
            }
            if let JsonValue::ValueObject(child_json_object) = &json_array[1] {
                member_assert_eq(&child_json_object.borrow(), "array_object_string", &JsonValue::ValueString("array_objct_string2".to_string()));
                member_assert_eq(&child_json_object.borrow(), "array_object_number", &JsonValue::ValueNumber(NumberType::Float(2.0)));
                assert_eq!(child_json_object.borrow().members.len(), 2);
            }
        }
        else {
            panic!();
        }
        assert_eq!(json_object.members.len(), 6);
        Ok(())
    }

    #[test]
    fn parse_object() -> Result<()> {
        let test_path = Path::new("./for_test/parse_test_object.json");
        let mut file = File::open(test_path)?;
        let mut content_string = String::new();
        file.read_to_string(&mut content_string)?;

        let json_object = JsonObject::parse(&content_string).unwrap();
        dbg!(json_object.clone());
        if let JsonValue::ValueObject(child_json_object) = json_object.members.get(&JsonKey("empty_object".to_string())).unwrap() {
            assert_eq!(child_json_object.borrow().members.len(), 0);
        }
        else {
            panic!();
        }
        if let JsonValue::ValueObject(child_json_object) = json_object.members.get(&JsonKey("object".to_string())).unwrap() {
            member_assert_eq(&child_json_object.borrow(), "object_string", &JsonValue::ValueString("object_string".to_string()));
            member_assert_eq(&child_json_object.borrow(), "object_number", &JsonValue::ValueNumber(NumberType::Int(1)));
            member_assert_eq(&child_json_object.borrow(), "object_bool", &JsonValue::ValueBool(true));
            member_assert_eq(&child_json_object.borrow(), "object_null", &JsonValue::ValueNull);
            member_assert_eq(&child_json_object.borrow(), "object_array", &JsonValue::ValueArray(vec![
                JsonValue::ValueNumber(NumberType::Int(1)), 
                JsonValue::ValueNumber(NumberType::Int(2)), 
                JsonValue::ValueNumber(NumberType::Int(3)), 
                ]));
            if let JsonValue::ValueObject(grand_child_json_object) = &child_json_object.borrow().members.get(&JsonKey("object_object".to_string())).unwrap() {
                member_assert_eq(&grand_child_json_object.borrow(), "object_object_string", &JsonValue::ValueString("object_object_string".to_string()));
                member_assert_eq(&grand_child_json_object.borrow(), "object_object_number", &JsonValue::ValueNumber(NumberType::Int(1)));
                assert_eq!(grand_child_json_object.borrow().members.len(), 2);
            }
            assert_eq!(child_json_object.borrow().members.len(), 6);
        }
        else {
            panic!();
        }
        assert_eq!(json_object.members.len(), 2);
        Ok(())
    }

    fn serialized_str_assert_eq(serialized_string: &str, object_brackets_lines: &(&str, &str), members_lines: &[&str]) {

        for (idx, line) in serialized_string.lines().enumerate() {
            let trimed_line = line.trim_end();
            if idx == 0 {
                assert_eq!(object_brackets_lines.0, trimed_line);
            }
            else if idx == members_lines.len() + 1 {
                assert_eq!(object_brackets_lines.1, trimed_line);
            }
            else if idx == members_lines.len() {
                members_lines.iter().position(|&expect_line| expect_line == trimed_line).unwrap();
            }
            else {
                members_lines.iter().position(|&expect_line| &(expect_line.to_string() + ",") == trimed_line).unwrap();
            }
        }
    }

    #[test]
    fn serialize_string() -> Result<()> {
        let object_brackets = ("{", "}");
        let member_lines = [
            r#"    "string" : "string""#,
            r#"    "escape_string" : "string""#,
            ];

        let mut json_object = JsonObject::new();

        let json_key = JsonKey("string".to_string());
        let json_value = JsonValue::ValueString("string".to_string());
        json_object.members.insert(json_key, json_value);

        let json_key = JsonKey("escape_string".to_string());
        let json_value = JsonValue::ValueString("string".to_string());
        json_object.members.insert(json_key, json_value);

        

        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Space(4)).unwrap();
        serialized_str_assert_eq(&serialized_string, &object_brackets,&member_lines);
        assert_eq!(serialized_string.lines().count(), 4);
        Ok(())
    }

    #[test]
    fn serialize_number() -> Result<()> {
        let object_brackets = ("{", "}");
        let member_lines = [
            r#"    "int" : 1"#,
            r#"    "minus_int" : -1"#,
            r#"    "float" : 0.1"#,
            r#"    "minus_float" : -0.1"#,
            ];

        let mut json_object = JsonObject::new();

        let json_key = JsonKey("int".to_string());
        let json_value = JsonValue::ValueNumber(NumberType::Int(1));
        json_object.members.insert(json_key, json_value);

        let json_key = JsonKey("minus_int".to_string());
        let json_value = JsonValue::ValueNumber(NumberType::Int(-1));
        json_object.members.insert(json_key, json_value);
        
        let json_key = JsonKey("float".to_string());
        let json_value = JsonValue::ValueNumber(NumberType::Float(0.1));
        json_object.members.insert(json_key, json_value);
        
        let json_key = JsonKey("minus_float".to_string());
        let json_value = JsonValue::ValueNumber(NumberType::Float(-0.1));
        json_object.members.insert(json_key, json_value);

        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Space(4)).unwrap();
        serialized_str_assert_eq(&serialized_string, &object_brackets,&member_lines);
        assert_eq!(serialized_string.lines().count(), 6);
        Ok(())
    }

    #[test]
    fn serialize_bool_null() -> Result<()> {
        let object_brackets = ("{", "}");
        let member_lines = [
            r#"    "true" : true"#,
            r#"    "false" : false"#,
            r#"    "null" : null"#,
            ];

        let mut json_object = JsonObject::new();

        let json_key = JsonKey("true".to_string());
        let json_value = JsonValue::ValueBool(true);
        json_object.members.insert(json_key, json_value);

        let json_key = JsonKey("false".to_string());
        let json_value = JsonValue::ValueBool(false);
        json_object.members.insert(json_key, json_value);
        
        let json_key = JsonKey("null".to_string());
        let json_value = JsonValue::ValueNull;
        json_object.members.insert(json_key, json_value);


        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Space(4)).unwrap();
        serialized_str_assert_eq(&serialized_string, &object_brackets,&member_lines);
        assert_eq!(serialized_string.lines().count(), 5);
        Ok(())
    }

    #[test]
    fn serialize_array() -> Result<()> {
        let object_brackets = ("{", "}");
        let member_lines = [
            r#"    "empty_array" : []"#,
            r#"    "array_string" : [ "string1", "string2", "string3"]"#,
            r#"    "array_number" : [ 1, 2, 3, 4, 5]"#,
            r#"    "array_bool_null" : [ true, false, null]"#,
            r#"    "array_array" : [ [ "0,0", "0,1"], [ "1,0", "1,1"]]"#,
            ];

        let mut json_object = JsonObject::new();

        let json_key = JsonKey("empty_array".to_string());
        let json_value = JsonValue::ValueArray(Vec::new());
        json_object.members.insert(json_key, json_value);

        let json_key = JsonKey("array_string".to_string());
        let json_value = JsonValue::ValueArray(vec![
            JsonValue::ValueString("string1".to_string()),
            JsonValue::ValueString("string2".to_string()),
            JsonValue::ValueString("string3".to_string()),
            ]);
        json_object.members.insert(json_key, json_value);
        
        let json_key = JsonKey("array_number".to_string());
        let json_value = JsonValue::ValueArray(vec![
            JsonValue::ValueNumber(NumberType::Int(1)),
            JsonValue::ValueNumber(NumberType::Int(2)),
            JsonValue::ValueNumber(NumberType::Int(3)),
            JsonValue::ValueNumber(NumberType::Int(4)),
            JsonValue::ValueNumber(NumberType::Int(5)),
            ]);
        json_object.members.insert(json_key, json_value);

        let json_key = JsonKey("array_bool_null".to_string());
        let json_value = JsonValue::ValueArray(vec![
            JsonValue::ValueBool(true),
            JsonValue::ValueBool(false),
            JsonValue::ValueNull,
            ]);
        json_object.members.insert(json_key, json_value);

        let json_key = JsonKey("array_array".to_string());
        let json_value = JsonValue::ValueArray(vec![
            JsonValue::ValueArray(vec![
                JsonValue::ValueString("0,0".to_string()),
                JsonValue::ValueString("0,1".to_string()),
            ]),
            JsonValue::ValueArray(vec![
                JsonValue::ValueString("1,0".to_string()),
                JsonValue::ValueString("1,1".to_string()),
            ]),
            ]);
        json_object.members.insert(json_key, json_value);


        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Space(4)).unwrap();
        serialized_str_assert_eq(&serialized_string, &object_brackets,&member_lines);
        assert_eq!(serialized_string.lines().count(), 7);
        Ok(())
    }

    
    #[test]
    fn serialize_object() -> Result<()> {
        // empty object
        let object_brackets = ("{", "}");
        let member_lines = [
            r#"    "empty_object" : {}"#,
            ];

        let mut json_object = JsonObject::new();

        let json_key = JsonKey("empty_object".to_string());
        let json_value = JsonValue::ValueObject(Rc::new(RefCell::new(JsonObject::new())));
        json_object.members.insert(json_key, json_value);


        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Space(4)).unwrap();
        serialized_str_assert_eq(&serialized_string, &object_brackets,&member_lines);
        assert_eq!(serialized_string.lines().count(), 3);

        // child object
        let mut json_child_object = JsonObject::new();
        let json_key = JsonKey("object_number".to_string());
        let json_value = JsonValue::ValueNumber(NumberType::Int(1));
        json_child_object.members.insert(json_key, json_value);

        let mut json_object = JsonObject::new();

        let json_key = JsonKey("object_object".to_string());
        let json_value = JsonValue::ValueObject(Rc::new(RefCell::new(json_child_object)));
        json_object.members.insert(json_key, json_value);

        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Space(4)).unwrap();

        let mut serialized_lines = serialized_string.lines();
        assert_eq!("{", serialized_lines.next().unwrap());
        assert_eq!(r#"    "object_object" : {"#, serialized_lines.next().unwrap());
        assert_eq!(r#"        "object_number" : 1"#, serialized_lines.next().unwrap());
        assert_eq!("    }", serialized_lines.next().unwrap());
        assert_eq!("}", serialized_lines.next().unwrap());

        assert_eq!(serialized_string.lines().count(), 5);
        Ok(())
    }

    #[test]
    fn new_line() -> Result<()> {
        // Lf
        let mut json_object = JsonObject::new();

        let json_key = JsonKey("Lf".to_string());
        let json_value = JsonValue::ValueObject(Rc::new(RefCell::new(JsonObject::new())));
        json_object.members.insert(json_key, json_value);


        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Space(4)).unwrap();
        assert_eq!("{\u{000a}    \"Lf\" : {}\u{000a}}\u{000a}", serialized_string);

        // CrLf
        let mut json_object = JsonObject::new();

        let json_key = JsonKey("CrLf".to_string());
        let json_value = JsonValue::ValueObject(Rc::new(RefCell::new(JsonObject::new())));
        json_object.members.insert(json_key, json_value);


        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::CrLf, crate::JsonSerializerIndentKind::Space(4)).unwrap();
        assert_eq!("{\u{000d}\u{000a}    \"CrLf\" : {}\u{000d}\u{000a}}\u{000d}\u{000a}", serialized_string);
        Ok(())
    }

    #[test]
    fn indent() -> Result<()> {
        // Tab
        let mut json_object = JsonObject::new();

        let json_key = JsonKey("Tab".to_string());
        let json_value = JsonValue::ValueObject(Rc::new(RefCell::new(JsonObject::new())));
        json_object.members.insert(json_key, json_value);


        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Tab).unwrap();
        assert_eq!("{\u{000a}\u{0009}\"Tab\" : {}\u{000a}}\u{000a}", serialized_string);

        // Space(0)
        let mut json_object = JsonObject::new();

        let json_key = JsonKey("Space(0)".to_string());
        let json_value = JsonValue::ValueObject(Rc::new(RefCell::new(JsonObject::new())));
        json_object.members.insert(json_key, json_value);


        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Space(0)).unwrap();
        assert_eq!("{\u{000a}\"Space(0)\" : {}\u{000a}}\u{000a}", serialized_string);

        // Space(10)
        let mut json_object = JsonObject::new();

        let json_key = JsonKey("Space(10)".to_string());
        let json_value = JsonValue::ValueObject(Rc::new(RefCell::new(JsonObject::new())));
        json_object.members.insert(json_key, json_value);


        let serialized_string = json_object.serialize(JsonSerializerNewLineKind::Lf, crate::JsonSerializerIndentKind::Space(10)).unwrap();
        assert_eq!("{\u{000a}          \"Space(10)\" : {}\u{000a}}\u{000a}", serialized_string);
        Ok(())
    }
    
    #[test]
    fn read_write() -> Result<()> {
        if !Path::new("./for_test/output/").exists() {
            create_dir(Path::new("./for_test/output/")).unwrap();
        }

        let test_path = Path::new("./for_test/read_test1.json");
        let mut file = File::open(test_path)?;
        let mut content_string = String::new();
        file.read_to_string(&mut content_string)?;

        match JsonObject::parse(&content_string) {
            Ok(json_object) => {
                //dbg!(json_object.clone());
                match json_object.serialize(crate::JsonSerializerNewLineKind::CrLf, crate::JsonSerializerIndentKind::Space(4)) {
                    Ok(content_string) => {
                        let json_path = Path::new("./for_test/output/write_test1.json");
                        let mut file = File::create(json_path)?;
                        file.write_all(content_string.as_bytes())?;
                    }
                    Err(err) => {
                        println!("Error!: {}", err);
                    }
                }
                
            }
            Err(err) => {
                println!("Error!: {}", err);
            }
        }

        Ok(())
    }

}
