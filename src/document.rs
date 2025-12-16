use crate::bytes::{count_leading_line_endings, line_ending, literal_to_u64};
use crate::constants::pdf_key::{START_XREF, TRAILER, XREF};
use crate::constants::{PREV, ROOT};
use crate::error::error_kind::{
    CANT_FIND_ROOT, EXCEPT_TRAILER, INVALID_PDF_FILE, NO_XREF_TABLE_FOUND,
};
use crate::error::{Result};
use crate::objects::{PDFNumber, PDFObject, XEntry};
use crate::parser::{parse, parse_text_xref};
use crate::sequence::{FileSequence, Sequence};
use crate::tokenizer::Tokenizer;
use crate::vpdf::PDFVersion;
use log::debug;
use std::path::PathBuf;
use crate::page::{create_page_tree_arena, PageTreeArean};

/// Represent a PDF document
pub struct PDFDocument {
    /// Cross-reference table
    xrefs: Vec<XEntry>,
    /// PDF version
    version: PDFVersion,
    /// Tokenizer
    tokenizer: Tokenizer,
    // Page Tree Arena
    page_tree_arena: PageTreeArean
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
        // Merge all xref table
        let (xrefs, catalog) = merge_xref_table(&mut tokenizer)?;
        let page_tree_arena = create_page_tree_arena(&mut tokenizer,catalog,&xrefs)?;
        let document = PDFDocument {
            xrefs,
            version,
            tokenizer,
            page_tree_arena
        };
        Ok(document)
    }
    /// Get xref slice
    pub fn get_xref_slice(&self) -> &[XEntry] {
        &self.xrefs
    }
    /// Find xref index
    pub fn find_xref_index<F>(&self, visit: F) -> Option<usize>
    where
        F: Fn(&XEntry) -> bool,
    {
        self.xrefs.iter().position(visit)
    }
    /// Get PDF version
    pub fn get_version(&self) -> &PDFVersion {
        &self.version
    }
    /// Read object from PDFDocument
    pub fn read_object(&mut self, index: usize) -> Result<Option<PDFObject>> {
        if index >= self.xrefs.len() {
            return Ok(None);
        }
        let entry = &self.xrefs[index];
        if entry.is_freed() {
            return Ok(None);
        }
        self.tokenizer.seek(entry.get_value())?;
        let object = parse(&mut self.tokenizer)?;
        Ok(Some(object))
    }

    /// Get pdf page number
    pub fn get_page_num(&self) -> usize {
        self.page_tree_arena.get_page_num()
    }
}

fn parse_version(sequence: &mut impl Sequence) -> Result<PDFVersion> {
    let mut buf = [0u8; 1024];
    let n = sequence.read(&mut buf)?;
    if n < 8 {
        return Err(INVALID_PDF_FILE.into());
    }
    if buf.len() < 8
        || buf[0] != b'%'
        || buf[1] != b'P'
        || buf[2] != b'D'
        || buf[3] != b'F'
        || buf[4] != b'-'
    {
        return Err(INVALID_PDF_FILE.into());
    }
    let version = String::from_utf8(buf[5..8].to_vec())?;
    Ok(version.try_into()?)
}

fn merge_xref_table(mut tokenizer: &mut Tokenizer) -> Result<(Vec<XEntry>, (u64, u64))> {
    let mut xrefs = Vec::<XEntry>::new();
    let mut root = None;
    loop {
        let is_xref = tokenizer.check_next_token0(false, |token| token.key_was(XREF))?;
        if !is_xref {
            return Err(NO_XREF_TABLE_FOUND.into());
        }
        let entries = parse_text_xref(tokenizer)?;
        if xrefs.is_empty() {
            xrefs.extend_from_slice(&entries);
        } else {
            for entry in entries {
                if let None = xrefs.iter().find(|it| it.obj_num == entry.obj_num) {
                    xrefs.push(entry);
                }
            }
        }
        if let PDFObject::Dict(mut dictionary) = parse(&mut tokenizer)? {
            if let Some(obj) = dictionary.remove(ROOT) {
                root = Some(obj);
            }
            if let Some(PDFObject::Number(PDFNumber::Unsigned(prev))) = dictionary.get(PREV) {
                tokenizer.seek(*prev)?;
                continue;
            }
            if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = root {
                return Ok((xrefs, (obj_num, gen_num)));
            }
            return Err(CANT_FIND_ROOT.into());
        }
        return Err(EXCEPT_TRAILER.into());
    }
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
                break;
            }
        }
    }
    // Can't find start xref
    if index == n {
        return Err(INVALID_PDF_FILE.into());
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
        return Err(INVALID_PDF_FILE.into());
    }
    let offset = literal_to_u64(&buf[start..end]);
    Ok(offset)
}
