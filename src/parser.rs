//! JSON Parser module.
use std::cell::RefCell;
use std::rc::Rc;

use super::{JsonKey, JsonNumberType, JsonObject, JsonValue};

use super::error::*;

fn parse_error(
    kind: JsonErrorKind,
    detail_str: &str,
    char_position: &CharPosition,
) -> Box<dyn std::error::Error + Send + Sync + 'static> {
    let (line, column) = char_position.get_position();
    return JsonError::new(kind, Some(format!("{} | line:{} column:{}", detail_str, line, column)));
}

#[derive(Clone, PartialEq)]
enum MemberParserStatus {
    StartObject,
    Key,
    Coron,
    Value,
    EndMember,
}

impl MemberParserStatus {
    fn new() -> MemberParserStatus {
        MemberParserStatus::StartObject
    }
}

#[derive(PartialEq)]
enum StartObjectKind {
    EmptyObject,
    HasSomeMember,
}

#[derive(Clone, PartialEq)]
enum EndMemberKind {
    EndMember,
    EndObject,
}

#[derive(Clone, PartialEq)]
enum ArraySeparatorKind {
    EndElement,
    EndArray,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct CharPosition {
    idx: usize,
    line: usize,
    first_idx_in_line: usize,
}

impl CharPosition {
    fn new() -> CharPosition {
        CharPosition {
            idx: 0,
            line: 0,
            first_idx_in_line: 0,
        }
    }

    fn increment(&mut self, unicode_char: &char) {
        self.idx += 1;
        if *unicode_char == '\n' {
            self.line += 1;
            self.first_idx_in_line = self.idx;
        }
    }

    fn get_idx(&self) -> usize {
        self.idx
    }

    fn get_position(&self) -> (usize, usize) {
        // 行数と文字数は1始まりなので+1して返す。
        (self.line + 1, self.idx - self.first_idx_in_line + 1)
    }
}

/// JSON parser struct.
#[derive(Clone, Debug)]
pub(crate) struct JsonParser {
    content_chars: Vec<char>,
    char_position: CharPosition,
}

impl JsonParser {
    /// Parse JSON function.
    #[allow(dead_code)]
    pub fn parse(content_str: &str) -> Result<JsonObject> {
        let mut json_parser = JsonParser::new(content_str);
        let res_json_object = json_parser.object_parser();
        json_parser.content_chars.clear();
        res_json_object
    }

    fn new(content_str: &str) -> JsonParser {
        JsonParser {
            content_chars: content_str.chars().collect(),
            char_position: CharPosition::new(),
        }
    }

    fn object_parser(&mut self) -> Result<JsonObject> {
        let mut json_object = JsonObject::new();
        let mut status: MemberParserStatus = MemberParserStatus::new();

        'in_object_loop: loop {
            let mut key: JsonKey = JsonKey(String::new());
            'in_member_loop: loop {
                match status {
                    MemberParserStatus::StartObject => match self.start_object_parser()? {
                        StartObjectKind::EmptyObject => {
                            status = MemberParserStatus::EndMember;
                        }
                        StartObjectKind::HasSomeMember => {
                            status = MemberParserStatus::Key;
                        }
                    },
                    MemberParserStatus::Key => {
                        key = self.key_parser()?;
                        status = MemberParserStatus::Coron;
                    }
                    MemberParserStatus::Coron => {
                        self.coron_parser()?;
                        status = MemberParserStatus::Value;
                    }
                    MemberParserStatus::Value => {
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
                if self.char_position.get_idx() == self.content_chars.len() {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInObject,
                        "Object is not closed.",
                        &self.char_position,
                    ));
                }
            }
        }
        Ok(json_object)
    }

    fn start_object_parser(&mut self) -> Result<StartObjectKind> {
        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                '{' => {
                    self.char_position.increment(unicode_char);
                    break;
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_position.increment(unicode_char);
                }
                _ => {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInObject,
                        "StartObject: Expected \'{\' but found an another character.",
                        &self.char_position,
                    ));
                }
            }
        }
        // 空オブジェクト判定処理
        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
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
                    self.char_position.increment(unicode_char);
                }
                _ => {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInObject,
                        "StartObject: Expected \'{\' but found an another character.",
                        &self.char_position,
                    ));
                }
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInObject,
            "StartObject: Object is not closed.",
            &self.char_position,
        ));
    }

    fn key_parser(&mut self) -> Result<JsonKey> {
        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                '\"' => {
                    return Ok(JsonKey(self.string_parser()?));
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_position.increment(unicode_char);
                }
                _ => {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInKey,
                        "Key: Expected \'\"\' but found an another character.",
                        &self.char_position,
                    ));
                }
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInKey,
            "Key: Object is not closed.",
            &self.char_position,
        ));
    }

    fn coron_parser(&mut self) -> Result<()> {
        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                ':' => {
                    self.char_position.increment(unicode_char);
                    return Ok(());
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_position.increment(unicode_char);
                }
                _ => {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInObject,
                        "Key: Expected \':\' but found an another character.",
                        &self.char_position,
                    ));
                }
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInObject,
            "Comma: Object is not closed.",
            &self.char_position,
        ));
    }

    fn value_parser(&mut self) -> Result<JsonValue> {
        // self.char_idx を更新しながらループを回すための2重ループ(loop、for)
        loop {
            for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
                match unicode_char {
                    '\"' => {
                        return Ok(JsonValue::ValueString(self.string_parser()?));
                    }
                    '-' | ('0'..='9') => {
                        return Ok(JsonValue::ValueNumber(self.number_parser()?));
                    }
                    't' | 'f' => {
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
                        return Err(parse_error(
                            JsonErrorKind::ParseErrorInValue,
                            "Value: Expected any charcter that start value but found an another character.",
                            &self.char_position,
                        ));
                    }
                }
            }
        }
    }

    fn end_member_parser(&mut self) -> Result<EndMemberKind> {
        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                '}' => {
                    self.char_position.increment(unicode_char);
                    return Ok(EndMemberKind::EndObject);
                }
                ',' => {
                    self.char_position.increment(unicode_char);
                    return Ok(EndMemberKind::EndMember);
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_position.increment(unicode_char);
                }
                _ => {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInObject,
                        "EndMember: Expected \'}\' or \',\' but found an another character.",
                        &self.char_position,
                    ));
                }
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInObject,
            "EndMember: Object is not closed.",
            &self.char_position,
        ));
    }

    // 連続で空白を処理するので、char_idxがその分増える。その前提で使う。
    fn blank_parser(&mut self) -> Result<()> {
        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_position.increment(unicode_char);
                }
                _ => return Ok(()),
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInObject,
            "Blank: Object is not closed.",
            &self.char_position,
        ));
    }

    fn string_parser(&mut self) -> Result<String> {
        let mut string: String = String::new();

        if self.char_position.get_idx() < self.content_chars.len() {
            if self.content_chars[self.char_position.get_idx()] != '\"' {
                return Err(parse_error(
                    JsonErrorKind::ParseErrorInString,
                    "String: Expected \'\"\' but found an another character.",
                    &self.char_position,
                ));
            }
        } else {
            return Err(parse_error(
                JsonErrorKind::ParseErrorInString,
                "String: Object is not closed.",
                &self.char_position,
            ));
        }
        self.char_position
            .increment(&self.content_chars[self.char_position.get_idx()]);

        // self.char_idx を更新しながらループを回すための2重ループ(loop、for)
        loop {
            for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
                match unicode_char {
                    '\"' => {
                        self.char_position.increment(unicode_char);
                        return Ok(string);
                    }
                    '\\' => {
                        string.push(self.escape_string_parser()?);
                        break;
                    }
                    _ => {
                        self.char_position.increment(unicode_char);
                        string.push(*unicode_char);
                    }
                }
            }
            if self.char_position.get_idx() == self.content_chars.len() {
                return Err(parse_error(
                    JsonErrorKind::ParseErrorInString,
                    "String: Object is not closed.",
                    &self.char_position,
                ));
            }
        }
    }

    fn escape_string_parser(&mut self) -> Result<char> {
        if self.char_position.get_idx() >= self.content_chars.len() {
            return Err(parse_error(
                JsonErrorKind::ParseErrorInString,
                "EscapeString: Object is not closed.",
                &self.char_position,
            ));
        }
        if '\\' != self.content_chars[self.char_position.get_idx()] {
            return Err(parse_error(
                JsonErrorKind::ParseErrorInString,
                "EscapeString: Expected \'\\\' but found an another character.",
                &self.char_position,
            ));
        }
        self.char_position
            .increment(&self.content_chars[self.char_position.get_idx()]);

        if self.char_position.get_idx() < self.content_chars.len() {
            let unicode_char = &self.content_chars[self.char_position.get_idx()];
            match unicode_char {
                '\"' | '\\' | '/' => {
                    self.char_position.increment(unicode_char);
                    return Ok(*unicode_char);
                }
                'b' => {
                    self.char_position.increment(unicode_char);
                    return Ok('\u{0008}');
                }
                'f' => {
                    self.char_position.increment(unicode_char);
                    return Ok('\u{000C}');
                }
                'n' => {
                    self.char_position.increment(unicode_char);
                    return Ok('\n');
                }
                'r' => {
                    self.char_position.increment(unicode_char);
                    return Ok('\r');
                }
                't' => {
                    self.char_position.increment(unicode_char);
                    return Ok('\t');
                }
                'u' => {
                    self.char_position.increment(unicode_char);
                    // 'uXXXX'の処理
                    return self.escape_string_utf16();
                }
                _ => {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInString,
                        "EscapeString: Expected any escaped character but found an another character.",
                        &self.char_position,
                    ));
                }
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInString,
            "EscapeString: Object is not closed.",
            &self.char_position,
        ));
    }

    fn escape_string_utf16(&mut self) -> Result<char> {
        let mut utf16_vec: Vec<u16> = Vec::new();

        loop {
            let mut unicode_hex: String = String::new();

            if utf16_vec.len() > 0 {
                if self.char_position.get_idx() + 1 >= self.content_chars.len() {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInString,
                        "EscapeString: Object is not closed.",
                        &self.char_position,
                    ));
                }
                if '\\' != self.content_chars[self.char_position.get_idx()]
                    || 'u' != self.content_chars[self.char_position.get_idx() + 1]
                {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInString,
                        "EscapeString: Expected \"\\u\" but found an another character.",
                        &self.char_position,
                    ));
                }
                self.char_position
                    .increment(&self.content_chars[self.char_position.get_idx()]);
                self.char_position
                    .increment(&self.content_chars[self.char_position.get_idx() + 1]);
            }

            for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
                if '0' <= *unicode_char && *unicode_char <= '9'
                    || 'a' <= *unicode_char && *unicode_char <= 'f'
                    || 'A' <= *unicode_char && *unicode_char <= 'F'
                {
                    self.char_position.increment(unicode_char);
                    unicode_hex.push(*unicode_char);
                } else {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInString,
                        "EscapeString: Expected any Hexadecimal character but found an another character.",
                        &self.char_position,
                    ));
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
                                return Err(parse_error(
                                    JsonErrorKind::ParseErrorInString,
                                    "EscapeString: Escaped string could not be parsed to \"char\".",
                                    &self.char_position,
                                ));
                            }
                        }
                    } else {
                        return Err(parse_error(
                            JsonErrorKind::ParseErrorInString,
                            "EscapeString: Escaped string could not be parsed to u32 value.",
                            &self.char_position,
                        ));
                    }
                }
            }
        }
    }

    fn number_parser(&mut self) -> Result<JsonNumberType> {
        let mut number_string: String = String::new();
        // '-'判定用
        let mut arrow_sign_char: bool = true;
        let mut is_exp_notation: bool = false;
        // 小数点の位置の判定用
        let mut digit_existed: bool = false;
        // integer or float 判定用
        let mut decimal_point_existed: bool = false;

        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                '-' => {
                    if arrow_sign_char {
                        self.char_position.increment(unicode_char);
                        number_string.push(*unicode_char);
                    } else {
                        return Err(parse_error(
                            JsonErrorKind::ParseErrorInNumber,
                            "Number: \'-\'s position is not allowed.",
                            &self.char_position,
                        ));
                    }
                    arrow_sign_char = false;
                }
                '+' => {
                    if arrow_sign_char && is_exp_notation {
                        self.char_position.increment(unicode_char);
                        number_string.push(*unicode_char);
                    } else {
                        return Err(parse_error(
                            JsonErrorKind::ParseErrorInNumber,
                            "Number: \'+\'s position is not allowed.",
                            &self.char_position,
                        ));
                    }
                    arrow_sign_char = false;
                }
                '.' => {
                    if digit_existed && !decimal_point_existed && !is_exp_notation {
                        self.char_position.increment(unicode_char);
                        number_string.push(*unicode_char);
                        decimal_point_existed = true;
                    } else {
                        return Err(parse_error(
                            JsonErrorKind::ParseErrorInNumber,
                            "Number: \'.\'s position is not allowed.",
                            &self.char_position,
                        ));
                    }
                }
                ('0'..='9') => {
                    self.char_position.increment(unicode_char);
                    number_string.push(*unicode_char);
                    digit_existed = true;
                    arrow_sign_char = false;
                }
                'e' | 'E' => {
                    if digit_existed {
                        self.char_position.increment(unicode_char);
                        is_exp_notation = true;
                        arrow_sign_char = true;
                        number_string.push(*unicode_char);
                    } else {
                        return Err(parse_error(
                            JsonErrorKind::ParseErrorInNumber,
                            "Number: \'e\' or \'E\'s position is not allowed.",
                            &self.char_position,
                        ));
                    }
                }
                ' ' | '\t' | '\n' | '\r' | ',' | '}' | ']' => {
                    if decimal_point_existed || is_exp_notation {
                        if let Ok(float_number) = number_string.parse::<f64>() {
                            return Ok(JsonNumberType::Float(float_number));
                        } else {
                            return Err(parse_error(
                                JsonErrorKind::ParseErrorInNumber,
                                "Number: Number string could not be parsed to \"f64\".",
                                &self.char_position,
                            ));
                        }
                    } else {
                        if let Ok(int_number) = number_string.parse::<i64>() {
                            return Ok(JsonNumberType::Int(int_number));
                        } else {
                            return Err(parse_error(
                                JsonErrorKind::ParseErrorInNumber,
                                "Number: Number string could not be parsed to \"i64\".",
                                &self.char_position,
                            ));
                        }
                    }
                }
                _ => {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInNumber,
                        "Number: Expected any number character but found an another character.",
                        &self.char_position,
                    ));
                }
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInNumber,
            "Number:  Object is not closed.",
            &self.char_position,
        ));
    }

    fn bool_parser(&mut self) -> Result<bool> {
        let mut bool_string: String = String::new();
        let is_string_true: bool = if self.content_chars[self.char_position.get_idx()] == 't' {
            true
        } else if self.content_chars[self.char_position.get_idx()] == 'f' {
            false
        } else {
            return Err(parse_error(
                JsonErrorKind::ParseErrorInBool,
                "Bool: Expected any bool character but found an another character.",
                &self.char_position,
            ));
        };

        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                ' ' | '\t' | '\n' | '\r' | ',' | '}' | ']' => {
                    if is_string_true {
                        if bool_string == "true" {
                            return Ok(is_string_true);
                        }
                    } else {
                        if bool_string == "false" {
                            return Ok(is_string_true);
                        }
                    }
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInBool,
                        "Bool: Expected \"true\" or \"false\" but found an another string.",
                        &self.char_position,
                    ));
                }
                _ => {
                    self.char_position.increment(unicode_char);
                    bool_string.push(*unicode_char);
                    if bool_string.len() > 5 {
                        return Err(parse_error(
                            JsonErrorKind::ParseErrorInBool,
                            "Bool: Expected \"true\" or \"false\" but found an too long string.",
                            &self.char_position,
                        ));
                    }
                }
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInBool,
            "Bool:  Object is not closed.",
            &self.char_position,
        ));
    }

    fn null_parser(&mut self) -> Result<()> {
        let mut null_string: String = String::new();

        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                ' ' | '\t' | '\n' | '\r' | ',' | '}' | ']' => {
                    if null_string == "null" {
                        return Ok(());
                    }
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInNull,
                        "Null: Expected \"null\" but found an another string.",
                        &self.char_position,
                    ));
                }
                _ => {
                    self.char_position.increment(unicode_char);
                    null_string.push(*unicode_char);
                    if null_string.len() > 4 {
                        return Err(parse_error(
                            JsonErrorKind::ParseErrorInNull,
                            "Null: Expected \"null\" but found an too long string.",
                            &self.char_position,
                        ));
                    }
                }
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInNull,
            "Null:  Object is not closed.",
            &self.char_position,
        ));
    }

    fn array_parser(&mut self) -> Result<Vec<JsonValue>> {
        if self.char_position.get_idx() >= self.content_chars.len() {
            return Err(parse_error(
                JsonErrorKind::ParseErrorInString,
                "Array: Object is not closed.",
                &self.char_position,
            ));
        }
        if '[' != self.content_chars[self.char_position.get_idx()] {
            return Err(parse_error(
                JsonErrorKind::ParseErrorInString,
                "Array: Expected \'[\' but found an another character.",
                &self.char_position,
            ));
        }
        self.char_position
            .increment(&self.content_chars[self.char_position.get_idx()]);

        let mut object_array: Vec<JsonValue> = Vec::new();
        let mut object_array_len: usize = object_array.len();

        // 空配列判定処理
        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                ']' => {
                    self.char_position.increment(unicode_char);
                    return Ok(object_array);
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_position.increment(unicode_char);
                }
                _ => {
                    break;
                }
            }
        }

        // self.char_idx を更新しながらループを回すための2重ループ(loop、for)
        'in_array_loop: loop {
            for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
                match unicode_char {
                    '\"' => {
                        object_array.push(JsonValue::ValueString(self.string_parser()?));
                        break;
                    }
                    '-' | ('0'..='9') => {
                        object_array.push(JsonValue::ValueNumber(self.number_parser()?));
                        break;
                    }
                    't' | 'f' => {
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
                        return Err(parse_error(
                            JsonErrorKind::ParseErrorInArray,
                            "Array: Expected any start member character but found an another character.",
                            &self.char_position,
                        ));
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
            if self.char_position.get_idx() == self.content_chars.len() {
                break 'in_array_loop;
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInArray,
            "Null:  Object is not closed.",
            &self.char_position,
        ));
    }

    fn array_separator_parser(&mut self) -> Result<ArraySeparatorKind> {
        for unicode_char in self.content_chars.iter().skip(self.char_position.get_idx()) {
            match unicode_char {
                ',' => {
                    self.char_position.increment(unicode_char);
                    return Ok(ArraySeparatorKind::EndElement);
                }
                ']' => {
                    self.char_position.increment(unicode_char);
                    return Ok(ArraySeparatorKind::EndArray);
                }
                ' ' | '\t' | '\n' | '\r' => {
                    self.char_position.increment(unicode_char);
                }
                _ => {
                    return Err(parse_error(
                        JsonErrorKind::ParseErrorInObject,
                        "Array: Expected \',\' but found an another character.",
                        &self.char_position,
                    ));
                }
            }
        }
        return Err(parse_error(
            JsonErrorKind::ParseErrorInObject,
            "Array: Object is not closed.",
            &self.char_position,
        ));
    }
}
