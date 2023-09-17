use std::rc::Rc;
use std::cell::RefCell;

use super::{JsonKey, JsonValue, JsonNumber, JsonObject};

use super::error::*;

fn parse_error(kind: JsonErrorKind, detail_str: &str, char_idx: usize) -> Box<dyn std::error::Error + Send + Sync + 'static> {
    return JsonError::new(kind, Some(format!("{} | char_idx:{}",detail_str,char_idx)));
}

#[derive(Clone, PartialEq)]
enum MemberParserStatus {
    StartObject, // '{'を探す
    Key, // Keyを探す
    Coron, // ':'を探す
    Value, // 値の処理
    EndMember, // ','または'}'または'},'を探す
}

impl MemberParserStatus {
    fn new() -> MemberParserStatus {
        MemberParserStatus::StartObject
    }
}

#[derive(PartialEq)]
enum StartObjectKind {
    EmptyObject,
    HasSomeMember
}

#[derive(Clone, PartialEq)]
enum EndMemberKind {
    EndMember,
    EndObject
}

#[derive(Clone, PartialEq)]
enum ArraySeparatorKind {
    EndElement,
    EndArray
}

#[derive(Clone, Debug)]
pub struct JsonParser {
    content_chars: Vec<char>,
    char_idx: usize,
}

impl JsonParser {
    #[allow(dead_code)]
    pub fn parse(content_str:  &str) -> Result<JsonObject>  {
        let mut json_parser = JsonParser::new(content_str);
        let res_json_object = json_parser.object_parser();
        json_parser.content_chars.clear();
        res_json_object
    }

    fn new<'a>(content_str:  &'a str) -> JsonParser{
        JsonParser {
            content_chars: content_str.chars().collect(),
            char_idx: 0,
        }
    }

    fn object_parser(&mut self) -> Result<JsonObject> {
        let mut json_object = JsonObject::new();
        let mut status: MemberParserStatus  = MemberParserStatus::new();

        'in_object_loop : loop {
            let mut key: JsonKey = JsonKey(String::new());
            'in_member_loop : loop {
                match status {
                    MemberParserStatus::StartObject => {
                        // '{'を探す
                        match self.start_object_parser()? {
                            StartObjectKind::EmptyObject => {
                                status = MemberParserStatus::EndMember;
                            }
                            StartObjectKind::HasSomeMember => {
                                status = MemberParserStatus::Key;
                            }
                        }
                    }
                    MemberParserStatus::Key => {
                        // Keyを探す
                        key = self.key_parser()?;
                        status = MemberParserStatus::Coron;
                    }
                    MemberParserStatus::Coron => {
                        // ':'を探す
                        self.coron_parser()?;
                        status = MemberParserStatus::Value;
                    }
                    MemberParserStatus::Value => {
                        // 値の処理
                        json_object.members.insert(key.clone(), self.value_parser()?);
                        status = MemberParserStatus::EndMember;
                    }
                    MemberParserStatus::EndMember => {
                        // ',' または '}' を探す。
                        // ',' なら次のオブジェクト内のメンバー、'}' ならオブジェクトが終わって親に帰る。
                        // (なお、"}," は '}' でオブジェクト終了したのち、 ',' で次のメンバー、という処理になる)
                        match self.end_member_parser()? {
                            EndMemberKind::EndMember => {
                                status = MemberParserStatus::Key;
                                break 'in_member_loop;
                            }
                            EndMemberKind::EndObject => {
                                break 'in_object_loop;
                            }
                        }
                    }
                    
                } 
                if self.char_idx == self.content_chars.len() {
                    return Err(parse_error(JsonErrorKind::ParseErrorInObject, "Object is not closed.", self.char_idx));
                }
            }
        }
        Ok(json_object)
    }

    fn start_object_parser(&mut self) -> Result<StartObjectKind> {
        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                '{' => {
                    self.char_idx += 1;
                    break;
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_idx += 1;
                }
                _ =>{
                    return Err(parse_error(JsonErrorKind::ParseErrorInObject, "StartObject: Expected \'{\' but found an another character.", self.char_idx));
                }
            }
        }
        // 空オブジェクト判定処理
        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                '}' => {
                    // '}' は end_member_parser で読み込みするためここではchar_idxの変更はなし。
                    return Ok(StartObjectKind::EmptyObject);
                }
                '\"' => {
                    // '\"' は key_parser で読み込みするためここではchar_idxの変更はなし。
                    return Ok(StartObjectKind::HasSomeMember);
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_idx += 1;
                }
                _ =>{
                    return Err(parse_error(JsonErrorKind::ParseErrorInObject, "StartObject: Expected \'{\' but found an another character.", self.char_idx));
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInObject, "StartObject: Object is not closed.", self.char_idx));
    }

    fn key_parser(&mut self) -> Result<JsonKey> {
        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                '\"' => {
                    return Ok(JsonKey(self.string_parser()?));
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_idx += 1;
                }
                _ => {
                    return Err(parse_error(JsonErrorKind::ParseErrorInKey, "Key: Expected \'\"\' but found an another character.", self.char_idx));
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInKey, "Key: Object is not closed.", self.char_idx));
    }

    fn coron_parser(&mut self) -> Result<()> {
        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                ':' => {
                    self.char_idx += 1;
                    return Ok(());
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_idx += 1;
                }
                _ =>{
                    return Err(parse_error(JsonErrorKind::ParseErrorInObject, "Key: Expected \':\' but found an another character.", self.char_idx));
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInObject, "Comma: Object is not closed.", self.char_idx));
    }

    fn value_parser(&mut self) -> Result<JsonValue> {
        // self.char_idx を更新しながらループを回すための2重ループ(loop、for)
        loop {
            for unicode_char in self.content_chars.iter().skip(self.char_idx) {
                match unicode_char {
                    '\"' => {
                        return Ok(JsonValue::ValueString(self.string_parser()?));
                    }
                    '-' | ('0'..='9')  => {
                        return Ok(JsonValue::ValueNumber(self.number_parser()?));
                    }
                    't' | 'f'  => {
                        return Ok(JsonValue::ValueBool(self.bool_parser()?));
                    }
                    'n' => {
                        self.null_parser()?;
                        return Ok(JsonValue::ValueNull);
                    }
                    '[' => {
                        return Ok(JsonValue::ValueArray(self.array_parser()?));
                    }
                    '{' => {
                        return Ok(JsonValue::ValueObject(Rc::new(RefCell::new(self.object_parser()?))));
                    }
                    ' ' | '\t' | '\n' | '\r' => {
                        self.blank_parser()?;
                        break;
                    }
                    _ => {
                        return Err(parse_error(JsonErrorKind::ParseErrorInValue, "Value: Expected any charcter that start value but found an another character.", self.char_idx));
                    }
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInValue, "Value: Object is not closed.", self.char_idx));
    }

    fn end_member_parser(&mut self) -> Result<EndMemberKind> {
        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                '}' => {
                    self.char_idx += 1;
                    return Ok(EndMemberKind::EndObject);
                }
                ',' => {
                    self.char_idx += 1;
                    return Ok(EndMemberKind::EndMember);
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_idx += 1;
                }
                _ =>{
                    return Err(parse_error(JsonErrorKind::ParseErrorInObject, "EndMember: Expected \'}\' or \',\' but found an another character.", self.char_idx));
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInObject, "EndMember: Object is not closed.", self.char_idx));
    }

    // 連続で空白を処理するので、char_idxがその分増える。その前提で使う。
    fn blank_parser(&mut self) -> Result<()> {
        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_idx += 1;
                }
                _ => {
                    return Ok(())
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInObject, "Blank: Object is not closed.", self.char_idx));
    }

    fn string_parser(&mut self) -> Result<String> {
        let mut string: String = String::new();

        if self.char_idx < self.content_chars.len() {
            if self.content_chars[self.char_idx] != '\"' {
                return Err(parse_error(JsonErrorKind::ParseErrorInString, "String: Expected \'\"\' but found an another character.", self.char_idx));
            }
        }
        else {
            return Err(parse_error(JsonErrorKind::ParseErrorInString, "String: Object is not closed.", self.char_idx));
        }
        self.char_idx += 1;

        // self.char_idx を更新しながらループを回すための2重ループ(loop、for)
        loop {
            for unicode_char in self.content_chars.iter().skip(self.char_idx) {
                match unicode_char {
                    '\"' => {
                        self.char_idx += 1;
                        return Ok(string);
                    }
                    '\\' => {
                        string.push(self.escape_string_parser()?);
                        break;
                    }
                    _ => {
                        self.char_idx += 1;
                        string.push(*unicode_char);
                    }
                }
            }
            if self.char_idx == self.content_chars.len() {
                return Err(parse_error(JsonErrorKind::ParseErrorInString, "String: Object is not closed.", self.char_idx));
            }
        }
    }

    

    fn escape_string_parser(&mut self) -> Result<char> {
        if self.char_idx >= self.content_chars.len() {
            return Err(parse_error(JsonErrorKind::ParseErrorInString, "EscapeString: Object is not closed.", self.char_idx));
        }
        if '\\' != self.content_chars[self.char_idx] {
            return Err(parse_error(JsonErrorKind::ParseErrorInString, "EscapeString: Expected \'\\\' but found an another character.", self.char_idx));
        }
        self.char_idx += 1;

        if self.char_idx < self.content_chars.len() {
            match self.content_chars[self.char_idx] {
                '\"' | '\\' | '/' => {
                    self.char_idx += 1;
                    return Ok(self.content_chars[self.char_idx]);
                }
                'b' => {
                    self.char_idx += 1;
                    return Ok('\u{0008}');
                }
                'f' => {
                    self.char_idx += 1;
                    return Ok('\u{000C}');
                }
                'n' => {
                    self.char_idx += 1;
                    return Ok('\n');
                }
                'r' => {
                    self.char_idx += 1;
                    return Ok('\r');
                }
                't' => {
                    self.char_idx += 1;
                    return Ok('\t');
                }
                'u' => {
                    self.char_idx += 1;
                    // 'uXXXX'の処理
                    return self.escape_string_utf16();
                }
                _ => {
                    return Err(parse_error(JsonErrorKind::ParseErrorInString, "EscapeString: Expected any escaped character but found an another character.", self.char_idx));
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInString, "EscapeString: Object is not closed.", self.char_idx));
    }

    fn escape_string_utf16(&mut self) -> Result<char> {
        let mut utf16_vec: Vec<u16> = Vec::new();

        loop {
            let mut unicode_hex: String = String::new();

            if utf16_vec.len() > 0 {
                if self.char_idx + 1 >= self.content_chars.len() {
                    return Err(parse_error(JsonErrorKind::ParseErrorInString, "EscapeString: Object is not closed.", self.char_idx));
                }
                if '\\' != self.content_chars[self.char_idx]
                || 'u' != self.content_chars[self.char_idx + 1 ] {
                    return Err(parse_error(JsonErrorKind::ParseErrorInString, "EscapeString: Expected \"\\u\" but found an another character.", self.char_idx));
                }
                self.char_idx += 2;
            }

            for unicode_char in self.content_chars.iter().skip(self.char_idx) {
                self.char_idx += 1;
                if '0' <= *unicode_char && *unicode_char <= '9'
                || 'a' <= *unicode_char && *unicode_char <= 'f'
                || 'A' <= *unicode_char && *unicode_char <= 'F'
                {
                    unicode_hex.push(*unicode_char);
                }
                else {
                    return Err(parse_error(JsonErrorKind::ParseErrorInString, "EscapeString: Expected any Hexadecimal character but found an another character.", self.char_idx));
                }
                if unicode_hex.len() == 4 {
                    if let Ok(u16_char_code) = u16::from_str_radix(&unicode_hex, 16) {
                        utf16_vec.push(u16_char_code);
                        if 0xD800 <= u16_char_code && u16_char_code <= 0xDBFF {
                            // サロゲートペアの処理のため一度ブレークして次の"\uxxxx"を取得する。
                            break;
                        }
                        // 一文字だけ処理（サロゲートペアも含まれる）
                        match char::decode_utf16(utf16_vec).next().unwrap() {
                            Ok(u32_char) => {
                                return Ok(u32_char);
                            }
                            Err(_) => {
                                return Err(parse_error(JsonErrorKind::ParseErrorInString, "EscapeString: Escaped string could not be parsed to \"char\".", self.char_idx));
                            }
                        }
                    }
                    else {
                        return Err(parse_error(JsonErrorKind::ParseErrorInString, "EscapeString: Escaped string could not be parsed to u32 value.", self.char_idx));
                    }
                }
            }
        }
    }

    fn number_parser(&mut self) -> Result<JsonNumber> {
        let mut number_string: String = String::new();
        // '-'判定用
        let mut arrow_sign_char: bool = true;
        let mut is_exp_notation: bool = false;
        // 小数点の位置の判定用
        let mut digit_existed: bool = false;
        // integer or float 判定用
        let mut decimal_point_existed: bool = false;

        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                '-'  => {
                    if arrow_sign_char {
                        self.char_idx += 1;
                        number_string.push(*unicode_char);
                    }
                    else {
                        return Err(parse_error(JsonErrorKind::ParseErrorInNumber, "Number: \'-\'s position is not allowed.", self.char_idx));
                    }
                    arrow_sign_char = false;
                }
                '+'  => {
                    if arrow_sign_char && is_exp_notation {
                        self.char_idx += 1;
                        number_string.push(*unicode_char);
                    }
                    else {
                        return Err(parse_error(JsonErrorKind::ParseErrorInNumber, "Number: \'+\'s position is not allowed.", self.char_idx));
                    }
                    arrow_sign_char = false;
                }
                '.'  => {
                    if digit_existed && !decimal_point_existed && !is_exp_notation {
                        self.char_idx += 1;
                        number_string.push(*unicode_char);
                        decimal_point_existed = true;
                    }
                    else {
                        return Err(parse_error(JsonErrorKind::ParseErrorInNumber, "Number: \'.\'s position is not allowed.", self.char_idx));
                    }
                }
                ('0'..='9')  => {
                    self.char_idx += 1;
                    number_string.push(*unicode_char);
                    digit_existed = true;
                    arrow_sign_char = false;
                }
                'e' | 'E'  => {
                    if digit_existed {
                        self.char_idx += 1;
                        is_exp_notation = true;
                        arrow_sign_char = true;
                        number_string.push(*unicode_char);
                    }
                    else {
                        return Err(parse_error(JsonErrorKind::ParseErrorInNumber, "Number: \'e\' or \'E\'s position is not allowed.", self.char_idx));
                    }
                }
                ' ' | '\t' | '\n' | '\r' | ',' | '}' | ']' => {
                    if decimal_point_existed || is_exp_notation {
                        if let Ok(float_number) = number_string.parse::<f64>() {
                            return Ok(JsonNumber::JsonFloat(float_number));
                        }
                        else {
                            return Err(parse_error(JsonErrorKind::ParseErrorInNumber, "Number: Number string could not be parsed to \"f64\".", self.char_idx));
                        }
                    }
                    else {
                        if let Ok(int_number) = number_string.parse::<i64>() {
                            return Ok(JsonNumber::JsonInt(int_number));
                        }
                        else {
                            return Err(parse_error(JsonErrorKind::ParseErrorInNumber, "Number: Number string could not be parsed to \"i64\".", self.char_idx));
                        }
                    }
                }
                _ => {
                    return Err(parse_error(JsonErrorKind::ParseErrorInNumber, "Number: Expected any number character but found an another character.", self.char_idx));
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInNumber, "Number:  Object is not closed.", self.char_idx));
    }

    fn bool_parser(&mut self) -> Result<bool> {
        let mut bool_string: String = String::new();
        let is_string_true: bool = if self.content_chars[self.char_idx] == 't' {
            true
        }
        else if self.content_chars[self.char_idx] == 'f' {
            false
        }
        else {
            return Err(parse_error(JsonErrorKind::ParseErrorInBool, "Bool: Expected any bool character but found an another character.", self.char_idx));
        };

        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                ' ' | '\t' | '\n' | '\r'  | ',' | '}' | ']' => {
                    if is_string_true {
                        if bool_string == "true" {
                            return Ok(is_string_true);
                        }
                    }
                    else {
                        if bool_string == "false" {
                            return Ok(is_string_true);
                        }
                    }
                    return Err(parse_error(JsonErrorKind::ParseErrorInBool, "Bool: Expected \"true\" or \"false\" but found an another string.", self.char_idx));
                }
                _ => {
                    self.char_idx += 1;
                    bool_string.push(*unicode_char);
                    if bool_string.len() > 5 {
                        return Err(parse_error(JsonErrorKind::ParseErrorInBool, "Bool: Expected \"true\" or \"false\" but found an too long string.", self.char_idx));
                    }
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInBool, "Bool:  Object is not closed.", self.char_idx));
    }

    fn null_parser(&mut self) -> Result<()> {
        let mut null_string: String = String::new();

        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                ' ' | '\t' | '\n' | '\r'  | ',' | '}'  | ']' => {
                    if null_string == "null" {
                        return Ok(());
                    }
                    return Err(parse_error(JsonErrorKind::ParseErrorInNull, "Null: Expected \"null\" but found an another string.", self.char_idx));
                }
                _ => {
                    self.char_idx += 1;
                    null_string.push(*unicode_char);
                    if null_string.len() > 4 {
                        return Err(parse_error(JsonErrorKind::ParseErrorInNull, "Null: Expected \"null\" but found an too long string.", self.char_idx));
                    }
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInNull, "Null:  Object is not closed.", self.char_idx));
    }

    fn array_parser(&mut self) -> Result<Vec<JsonValue>> {
        if self.char_idx >= self.content_chars.len() {
            return Err(parse_error(JsonErrorKind::ParseErrorInString, "Array: Object is not closed.", self.char_idx));
        }
        if '[' != self.content_chars[self.char_idx] {
            return Err(parse_error(JsonErrorKind::ParseErrorInString, "Array: Expected \'[\' but found an another character.", self.char_idx));
        }
        self.char_idx += 1;

        let mut object_array : Vec<JsonValue> = Vec::new();
        let mut object_array_len: usize = object_array.len();

        // 空配列判定処理
        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                ']' => {
                    self.char_idx += 1;
                    return Ok(object_array);
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_idx += 1;
                }
                _ =>{
                    break;
                }
            }
        }


        // self.char_idx を更新しながらループを回すための2重ループ(loop、for)
        'in_array_loop : loop {
            for unicode_char in self.content_chars.iter().skip(self.char_idx) {
                match unicode_char {
                    '\"' => {
                        object_array.push(JsonValue::ValueString(self.string_parser()?));
                        break;
                    }
                    '-' | ('0'..='9')  => {
                        object_array.push(JsonValue::ValueNumber(self.number_parser()?));
                        break;
                    }
                    't' | 'f'  => {
                        object_array.push(JsonValue::ValueBool(self.bool_parser()?));
                        break;
                    }
                    'n' => {
                        self.null_parser()?;
                        object_array.push(JsonValue::ValueNull);
                        break;
                    }
                    '[' => {
                        object_array.push(JsonValue::ValueArray(self.array_parser()?));
                        break;
                    }
                    '{' => {
                        object_array.push(JsonValue::ValueObject(Rc::new(RefCell::new(self.object_parser()?))));
                        break;
                    }
                    ' ' | '\t' | '\n' | '\r' => {
                        self.blank_parser()?;
                        break;
                    }
                    _ => {
                        return Err(parse_error(JsonErrorKind::ParseErrorInArray, "Array: Expected any start member character but found an another character.", self.char_idx));
                    }
                }
            }
            if object_array.len() > object_array_len {
                object_array_len = object_array.len();
                match self.array_separator_parser()? {
                    ArraySeparatorKind::EndElement => {
                        // Nothing to do (Go to Next element)
                    }
                    ArraySeparatorKind::EndArray => {
                        return Ok(object_array);
                    }
                }
            }
            if self.char_idx == self.content_chars.len() {
                break 'in_array_loop;
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInArray, "Null:  Object is not closed.", self.char_idx));
    }

    fn array_separator_parser(&mut self) -> Result<ArraySeparatorKind> {
        for unicode_char in self.content_chars.iter().skip(self.char_idx) {
            match unicode_char {
                ',' => {
                    self.char_idx += 1;
                    return Ok(ArraySeparatorKind::EndElement);
                }
                ']' => {
                    self.char_idx += 1;
                    return Ok(ArraySeparatorKind::EndArray);
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_idx += 1;
                }
                _ =>{
                    return Err(parse_error(JsonErrorKind::ParseErrorInObject, "Array: Expected \',\' but found an another character.", self.char_idx));
                }
            }
        }
        return Err(parse_error(JsonErrorKind::ParseErrorInObject, "Array: Object is not closed.", self.char_idx));
    }
}
