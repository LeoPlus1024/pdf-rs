use std::num::ParseIntError;
use std::string::FromUtf8Error;
use crate::error::error_kind::{INT_PARSE_ERROR, INVALID_UTF8_STR, STD_IO_ERROR};

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
    (EOF, 1004, "EOF"),
    (INVALID_UTF8_STR,1005, "Invalid UTF8 string"),
    (INT_PARSE_ERROR,1006, "Int parse error"),
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
    fn from(value: FromUtf8Error) -> Self {
        Self {
            inner: Inner {
                code: INVALID_UTF8_STR.0,
                message: value.to_string(),
            },
        }
    }
}


impl From<ParseIntError> for Error{
    fn from(value: ParseIntError) -> Self {
        Self {
            inner: Inner {
                code: INT_PARSE_ERROR.0,
                message: value.to_string(),
            },
        }
    }
}