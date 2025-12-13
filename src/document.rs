use crate::constants::pdf_key::{START_XREF, TRAILER, XREF};
use crate::error::error_kind::INVALID_PDF_FILE;
use crate::error::Result;
use crate::objects::{PDFObject, PDFXref};
use crate::parser::parse;
use crate::sequence::{FileSequence, Sequence};
use crate::tokenizer::Tokenizer;
use crate::vpdf::PDFVersion;
use std::path::PathBuf;
use log::debug;
use crate::bytes::{count_leading_line_endings, line_ending, literal_to_u64};

/// Represent a PDF document
pub struct PDFDocument {
    /// Cross-reference table
    xrefs: Vec<PDFXref>,
    /// PDF version
    version: PDFVersion,
    // Tokenizer
    tokenizer: Tokenizer
}

impl PDFDocument {

    /// Open a pdf document
    pub fn open(path: PathBuf) -> Result<PDFDocument> {
        let file = std::fs::File::open(path)?;
        let sequence = FileSequence::new(file);
        Self::new(sequence)
    }

    /// Create a pdf document from sequence
    pub fn new(mut sequence: impl Sequence + 'static) -> Result<PDFDocument> {
        let version = parse_version(&mut sequence)?;
        let offset = cal_xref_table_offset(&mut sequence)?;
        let mut tokenizer = Tokenizer::new(sequence);
        tokenizer.seek(offset)?;
        let xrefs = parse_xref(&mut tokenizer)?;
        let document = PDFDocument {
            xrefs,
            version,
            tokenizer,
        };
        Ok(document)
    }
    pub fn get_xref(&self) -> &Vec<PDFXref> {
        &self.xrefs
    }
    pub fn get_version(&self) -> &PDFVersion {
        &self.version
    }
}

fn parse_version(sequence: &mut impl Sequence) -> Result<PDFVersion> {
    let mut buf = [0u8; 1024];
    let n = sequence.read(&mut buf)?;
    if n < 8 {
        return Err(INVALID_PDF_FILE.into());
    }
    if buf.len() < 8
        || buf[0] != 37
        || buf[1] != 80
        || buf[2] != 68
        || buf[3] != 70
        || buf[4] != 45
    {
        return Err(INVALID_PDF_FILE.into());
    }
    let version = String::from_utf8(buf[5..8].to_vec())?;
    Ok(version.try_into()?)
}

fn parse_xref(mut tokenizer: &mut Tokenizer) -> Result<Vec<PDFXref>> {
    if let Some(PDFObject::Xref(xref)) = parse(&mut tokenizer, |token| token.key_was(XREF))? {
        if let Some(PDFObject::Dict(dict)) = parse(&mut tokenizer, |token| token.key_was(TRAILER))? {
            return Ok(vec![xref])
        }
    }
    Ok(vec![])
}

fn cal_xref_table_offset(sequence: &mut impl Sequence) -> Result<u64> {
    let size = sequence.size()?;
    let pos = if size > 1024 { size - 1024 } else { 0 };
    let mut buf = [0u8; 1024];
    sequence.seek(pos)?;
    let n = sequence.read(&mut buf)?;
    let chars = START_XREF.as_bytes();
    let mut tx = chars.len();
    let mut index = n;
    for i in (0..n).rev() {
        let b = buf[i];
        if chars[tx - 1] == b {
            tx -= 1;
            if tx == 0 {
                index = i;
                break
            }
        }
    }
    // Can't find start xref
    if index == n {
        return Err(INVALID_PDF_FILE.into())
    }
    index = index + chars.len();
    let crlf_num = count_leading_line_endings(&buf[index..n]);
    let start = index + (crlf_num as usize);
    let mut end = 0usize;
    for i in start..n {
        if line_ending(buf[i]) {
            end = i;
            break;
        }
    }
    if end == 0 || start == end {
        debug!("Start-Xref offset not normal end");
        return Err(INVALID_PDF_FILE.into())
    }
    let offset = literal_to_u64(&buf[start..end]);
    Ok(offset)
}
