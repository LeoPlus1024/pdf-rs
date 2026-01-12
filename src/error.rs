use std::num::{ParseFloatError, ParseIntError};
use std::string::FromUtf8Error;
use thiserror::Error;

/// Type alias for results that may contain errors.
pub type Result<T> = std::result::Result<T, PDFError>;

#[derive(Error, Debug)]
pub enum PDFError {
    #[error("Not support pdf version:{0}")]
    NotSupportPDFVersion(String),
    #[error("Invalid PDF document.")]
    InvalidPDFDocument,
    #[error("Xref table not found.")]
    XrefTableNotFound,
    #[error("Convert utf8 text error:{0}")]
    UTF8Error(#[from] FromUtf8Error),
    #[error("IO error:{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    PDFParseError(&'static str),
    #[error("{0}")]
    PDFParseError0(String),
    #[error("Xref entry:({0},{1}) not found")]
    XrefEntryNotFound(u32,u16),
    #[error("{0}")]
    ObjectAttrMiss(&'static str),
    #[error("End of file error")]
    EOFError,
    #[error("Seek exceed maximum of file size")]
    SeekExceedError,
    #[error("{0}")]
    IntParseError(#[from] ParseIntError),
    #[error("{0}")]
    FloatParseError(#[from] ParseFloatError),
    #[error("{0}")]
    PDFObjectCastError(&'static str),
    #[error("Illegal date format:{0}")]
    IllegalDateFormat(String),
}
