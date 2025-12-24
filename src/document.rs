use crate::catalog::{decode_catalog_data, OutlineTreeArean, PageTreeArean};
use crate::constants::pdf_key::{START_XREF, XREF};
use crate::constants::{INFO, PREV, ROOT};
use crate::error::PDFError::{InvalidPDFDocument, ObjectAttrMiss, PDFParseError, XrefTableNotFound};
use crate::error::Result;
use crate::objects::{PDFNumber, PDFObject, XEntry};
use crate::parser::{parse, parse_text_xref, parse_with_offset};
use crate::sequence::{FileSequence, Sequence};
use crate::tokenizer::Tokenizer;
use crate::utils::{count_leading_line_endings, line_ending, literal_to_u64, xrefs_search};
use std::path::PathBuf;
use crate::vpdf::PDFVersion;

pub struct PDFDescribe {

}

/// Represents a PDF document with all its components and functionality.
///
/// This struct encapsulates a parsed PDF document, providing access to its cross-reference
/// table, version information, tokenizer, and page structure.
pub struct PDFDocument {
    /// Cross-reference table containing references to all objects in the PDF.
    xrefs: Vec<XEntry>,
    /// PDF version information.
    version: PDFVersion,
    /// Tokenizer for parsing the PDF content.
    tokenizer: Tokenizer,
    /// Page tree arena containing the hierarchical page structure.
    page_tree_arena: PageTreeArean,
    /// Outline tree arena containing the hierarchical outline structure.
    outline_tree_arean: Option<OutlineTreeArean>,
    /// Document info
    describe: Option<PDFDescribe>,
}

impl PDFDocument {
    /// Opens a PDF document from a file path.
    ///
    /// This function opens a PDF file, reads its content, and parses it into a PDFDocument.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the PDF file to open
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `PDFDocument` or an error if the file cannot be opened
    /// or parsed correctly
    pub fn open(path: PathBuf) -> Result<PDFDocument> {
        let file = std::fs::File::open(path)?;
        let sequence = FileSequence::new(file);
        Self::new(sequence)
    }

    /// Creates a PDF document from a sequence of bytes.
    ///
    /// This function parses a sequence of bytes representing a PDF document and constructs
    /// a PDFDocument instance with all its components.
    ///
    /// # Arguments
    ///
    /// * `sequence` - A sequence implementation providing access to the PDF bytes
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `PDFDocument` or an error if parsing fails
    pub fn new(mut sequence: impl Sequence + 'static) -> Result<PDFDocument> {
        let version = parse_version(&mut sequence)?;
        let offset = cal_xref_table_offset(&mut sequence)?;
        let mut tokenizer = Tokenizer::new(sequence);
        tokenizer.seek(offset)?;
        // Merge all xref table
        let (xrefs, catalog,info) = merge_xref_table(&mut tokenizer)?;
        let (page_tree_arena, outline_tree_arean) = match catalog {
            Some(catalog) => decode_catalog_data(&mut tokenizer, catalog, &xrefs)?,
            None => return Err(ObjectAttrMiss("Trailer can't found catalog attr.")),
        };
        let mut describe = None;
        // Parse document info
        if let Some(obj) = info {
            let entry = xrefs_search(&xrefs, obj)?;
            if let PDFObject::IndirectObject(_, _, value) = parse_with_offset(&mut tokenizer, entry.value)? {
                if let PDFObject::Dict(dict) = *value {
                    describe = Some(PDFDescribe::new());
                }
            }
        }
        let document = PDFDocument {
            xrefs,
            version,
            tokenizer,
            page_tree_arena,
            outline_tree_arean,
            describe,
        };
        Ok(document)
    }

    /// Gets a reference to the cross-reference table slice.
    ///
    /// # Returns
    ///
    /// A slice reference to the vector of cross-reference entries
    pub fn get_xref_slice(&self) -> &[XEntry] {
        &self.xrefs
    }

    /// Finds the index of a cross-reference entry that matches a condition.
    ///
    /// # Arguments
    ///
    /// * `visit` - A closure that takes a reference to an XEntry and returns a boolean
    ///
    /// # Returns
    ///
    /// An optional index of the first matching entry, or None if no entry matches
    pub fn find_xref_index<F>(&self, visit: F) -> Option<usize>
    where
        F: Fn(&XEntry) -> bool,
    {
        self.xrefs.iter().position(visit)
    }

    /// Gets the PDF version information.
    ///
    /// # Returns
    ///
    /// A reference to the PDFVersion struct containing version information
    pub fn get_version(&self) -> &PDFVersion {
        &self.version
    }

    /// Reads an object from the PDF document by its index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the object to read from the cross-reference table
    ///
    /// # Returns
    ///
    /// A `Result` containing an optional PDFObject (None if the index is out of bounds
    /// or the object is freed) or an error if reading/parsing fails
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

    /// Gets the total number of pages in the PDF document.
    ///
    /// # Returns
    ///
    /// The number of pages in the document
    pub fn get_page_num(&self) -> usize {
        self.page_tree_arena.get_page_num()
    }
}

/// Parses the PDF version from the beginning of the document.
///
/// This function reads the first few bytes of a PDF document to extract and validate
/// the PDF version information.
///
/// # Arguments
///
/// * `sequence` - A mutable reference to a sequence implementation for reading bytes
///
/// # Returns
///
/// A `Result` containing the parsed PDFVersion or an error if the version cannot be
/// parsed or is invalid
fn parse_version(sequence: &mut impl Sequence) -> Result<PDFVersion> {
    let mut buf = [0u8; 1024];
    let n = sequence.read(&mut buf)?;
    if n < 8 {
        return Err(InvalidPDFDocument);
    }
    if buf.len() < 8
        || buf[0] != b'%'
        || buf[1] != b'P'
        || buf[2] != b'D'
        || buf[3] != b'F'
        || buf[4] != b'-'
    {
        return Err(InvalidPDFDocument);
    }
    let version = String::from_utf8(buf[5..8].to_vec())?;
    Ok(version.try_into()?)
}

/// Merges cross-reference tables from a PDF document.
///
/// This function parses and merges multiple cross-reference tables that may exist
/// in a PDF document, handling cases where there are previous xref tables referenced
/// in the document trailer.
///
/// # Arguments
///
/// * `tokenizer` - A mutable reference to the tokenizer for parsing PDF content
///
/// # Returns
///
/// A `Result` containing a tuple with the merged vector of XEntry objects and
/// a tuple of the catalog object number and generation number, or an error if
/// parsing fails
fn merge_xref_table(mut tokenizer: &mut Tokenizer) -> Result<(Vec<XEntry>, Option<(u32, u16)>, Option<(u32, u16)>)> {
    let mut xrefs = Vec::<XEntry>::new();
    let mut info = None;
    let mut catalog = None;
    loop {
        let is_xref = tokenizer.check_next_token0(false, |token| token.key_was(XREF))?;
        if !is_xref {
            return Err(XrefTableNotFound);
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
            if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = dictionary.get(ROOT) {
                catalog = Some((*obj_num, *gen_num));
                if let Some(PDFObject::ObjectRef(obj_num, gen_num)) = dictionary.get(INFO)
                {
                    info = Some((*obj_num, *gen_num));
                }
            }
            // Recursive previous xref
            if let Some(PDFObject::Number(PDFNumber::Unsigned(prev))) = dictionary.get(PREV) {
                tokenizer.seek(*prev)?;
                continue;
            }
            return Ok((xrefs, catalog, info));
        }
        return Err(PDFParseError("Xref table broken."));
    }
}

/// Calculates the offset of the cross-reference table in the PDF document.
///
/// This function searches for the "startxref" keyword near the end of the document
/// and extracts the offset value that points to the beginning of the cross-reference table.
///
/// # Arguments
///
/// * `sequence` - A mutable reference to a sequence implementation for reading bytes
///
/// # Returns
///
/// A `Result` containing the calculated offset as a u64 value, or an error if the
/// startxref keyword cannot be found or the offset cannot be parsed
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
        return Err(InvalidPDFDocument);
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
        return Err(InvalidPDFDocument);
    }
    let offset = literal_to_u64(&buf[start..end]);
    Ok(offset)
}

impl PDFDescribe {
    pub(crate) fn new() -> PDFDescribe {
        PDFDescribe {}
    }
}