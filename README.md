# json_parser_serializer
JSON parser &amp; serializer library for Rust

## Documents
```
cargo doc --open
```

## Usage Examples
### Parse
```Rust
extern crate json_parser_serializer;
use json_parser_serializer::{JsonObject, JsonKey, JsonValue};

fn main() {
    let json_content_str: &str = "{\r    \"usage\" : \"usage string\"\r}";

    // Parse
    let json_object = JsonObject::parse(&json_content_str).unwrap();

    let json_key = JsonKey("usage".to_string());
    let json_value = json_object.members.get(&json_key).unwrap();
    match json_value {
        JsonValue::ValueString(value_string) => {
            println!("key:{}, value:{}", json_key.0, value_string);
        }
        _ => {
            println!("Error!");
        }
    }
}
```

### Serialize
```Rust
extern crate json_parser_serializer;
use json_parser_serializer::{JsonObject, JsonKey, JsonValue, JsonSerializerNewLineKind, JsonSerializerIndentKind};

fn main() {
    // Create Object
    let mut json_object = JsonObject::new();

    let json_key = JsonKey("usage".to_string());
    let json_value = JsonValue::ValueString("usage string".to_string());
    json_object.members.insert(json_key, json_value);

    // Serialize
    let json_string = json_object.serialize(JsonSerializerNewLineKind::Lf, JsonSerializerIndentKind::Space(4)).unwrap();
    
    println!("{}", json_string);
}
```