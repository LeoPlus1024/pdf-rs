use std::fmt::Display;
use std::num::{ParseFloatError, ParseIntError};
use std::string::FromUtf8Error;
use crate::error::error_kind::{FLOAT_PARSE_ERROR, INT_PARSE_ERROR, INVALID_UTF8_STR, STD_IO_ERROR};

macro_rules! error_kind {
    ($(($id:ident,$code:literal,$message:literal)),+$(,)?) => {
        pub(crate) mod error_kind{
        $(
            pub(crate) const $id: super::Kind = ($code, $message);
        )+
    }
    };
}

pub(crate) type Kind = (u16, &'static str);

pub type Result<T> = std::result::Result<T, Error>;

error_kind!(
    (INVALID_PDF_VERSION, 1000, "Invalid PDF version"),
    (STD_IO_ERROR, 1001, "Std IO Error"),
    (INVALID_PDF_FILE, 1002, "Invalid PDF file"),
    (TRAILER_NOT_FOUND, 1003, "Trailer not found"),
    (EOF, 1004, "End of file"),
    (INVALID_UTF8_STR,1005, "Invalid UTF8 string"),
    (INT_PARSE_ERROR,1006, "Int parse error"),
    (INVALID_CROSS_TABLE_ENTRY,1007, "Invalid cross table entry"),
    (TRAILER_EXCEPT_A_DICT,1008, "Trailer except a dict"),
    (INVALID_NUMBER,1009, "Invalid number"),
    (FLOAT_PARSE_ERROR,1010, "Float parse error"),
    (UNKNOWN_TOKEN,1011, "Unknown token"),
    (EXCEPT_TOKEN,1012, "Except a token"),
    (STR_NOT_ENCODED,1013, "String not encoded"),
    (ILLEGAL_TOKEN,1014, "Illegal token"),
    (INVALID_REAL_NUMBER,1015, "Invalid real number"),
);

#[derive(Debug)]
struct Inner {
    pub code: u16,
    pub message: String,
}

#[derive(Debug)]
pub struct Error {
    inner: Inner,
}

impl Error {
    pub(crate) fn new(kind: Kind, message: String) -> Self {
        Self {
            inner: Inner {
                code: kind.0,
                message,
            },
        }
    }
}

impl From<Kind> for Error {
    fn from(kind: Kind) -> Self {
        Self {
            inner: Inner {
                code: kind.0,
                message: kind.1.to_string(),
            },
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self {
            inner: Inner {
                code: STD_IO_ERROR.0,
                message: e.to_string(),
            },
        }
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Self {
            inner: Inner {
                code: INVALID_UTF8_STR.0,
                message: e.to_string(),
            },
        }
    }
}


impl From<ParseIntError> for Error{
    fn from(e: ParseIntError) -> Self {
        Self {
            inner: Inner {
                code: INT_PARSE_ERROR.0,
                message: e.to_string(),
            },
        }
    }
}

impl From<ParseFloatError> for Error {
    fn from(e: ParseFloatError) -> Self {

        Self {
            inner: Inner {
                code: FLOAT_PARSE_ERROR.0,
                message: e.to_string(),
            },
        }
    }
}