use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[derive(Debug, Clone, PartialEq)]
pub struct JsonError {
    pub err_kind: JsonErrorKind,
    pub op_additional_message: Option<String>,
}

impl JsonError {
    pub fn new(
        err_kind: JsonErrorKind,
        op_additional_message: Option<String>,
    ) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        Box::<JsonError>::new(JsonError {
            err_kind: err_kind,
            op_additional_message: op_additional_message,
        })
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        for err_message in JSON_ERR_MESSAGE {
            if err_message.err_kind == self.err_kind {
                if let Some(additional_message) = &self.op_additional_message {
                    return write!(f, "{}", format!("{} : {}", err_message.message, additional_message));
                } else {
                    return write!(f, "{}", format!("{}", err_message.message));
                }
            }
        }
        panic!()
    }
}

impl std::error::Error for JsonError {}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum JsonErrorKind {
    ParseErrorInObject,
    ParseErrorInKey,
    ParseErrorInValue,
    ParseErrorInString,
    ParseErrorInNumber,
    ParseErrorInBool,
    ParseErrorInNull,
    ParseErrorInArray,
    SerializeErrorInObject,
    SerializeErrorInKey,
    SerializeErrorInValue,
    SerializeErrorInString,
    SerializeErrorInNumber,
    SerializeErrorInBool,
    SerializeErrorInNull,
    SerializeErrorInArray,
}

struct JsonErrorMessage {
    err_kind: JsonErrorKind,
    message: &'static str,
}

const JSON_ERR_MESSAGE: [JsonErrorMessage; 16] = [
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInObject,
        message: "Parse error in object",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInKey,
        message: "Parse error in key",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInValue,
        message: "Parse error in value",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInString,
        message: "Parse error in string",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInNumber,
        message: "Parse error in number",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInBool,
        message: "Parse error in bool",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInNull,
        message: "Parse error in null",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInArray,
        message: "Parse error in array",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::SerializeErrorInObject,
        message: "Serialize error in object",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::SerializeErrorInKey,
        message: "Serialize error in key",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::SerializeErrorInValue,
        message: "Serialize error in value",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInString,
        message: "Serialize error in string",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::SerializeErrorInNumber,
        message: "Serialize error in number",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::SerializeErrorInBool,
        message: "Serialize error in bool",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::ParseErrorInNull,
        message: "Serialize error in null",
    },
    JsonErrorMessage {
        err_kind: JsonErrorKind::SerializeErrorInArray,
        message: "Serialize error in array",
    },
];
